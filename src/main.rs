use eyre::OptionExt;
use wgpu::util::DeviceExt;

async fn run() -> eyre::Result<()> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .ok_or_eyre("request adapter")?;
    let features = adapter.features();
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: features | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
                ..Default::default()
            },
            None,
        )
        .await?;
    let query_set = features
        .contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS)
        .then(|| {
            device.create_query_set(&wgpu::QuerySetDescriptor {
                label: None,
                ty: wgpu::QueryType::Timestamp,
                count: 2,
            })
        });

    let cs_module = {
        let _span_guard = tracing::info_span!("shader compilation");
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        })
    };
    let input_f: Vec<_> = (0..1_000_000).map(|x| x as f32).collect();
    let input = bytemuck::cast_slice(&input_f);
    let input_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: input,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
    });
    let output_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: input.len() as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let query_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 16,
        usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let query_staging_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 16,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        module: &cs_module,
        entry_point: None,
        compilation_options: Default::default(),
        cache: None,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &input_buf,
                offset: 0,
                size: None,
            }),
        }],
    });

    let mut encoder = device.create_command_encoder(&Default::default());
    if let Some(query_set) = &query_set {
        encoder.write_timestamp(query_set, 0);
    }
    {
        let mut cpass = encoder.begin_compute_pass(&Default::default());
        cpass.set_pipeline(&pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups((input_f.len() as u32 + 256 - 1) / 256, 1, 1);
    }
    if let Some(query_set) = &query_set {
        encoder.write_timestamp(query_set, 1);
    }
    encoder.copy_buffer_to_buffer(&input_buf, 0, &output_buf, 0, input.len() as u64);
    if let Some(query_set) = &query_set {
        encoder.resolve_query_set(query_set, 0..2, &query_buf, 0);
    }
    encoder.copy_buffer_to_buffer(&query_buf, 0, &query_staging_buf, 0, 16);
    queue.submit(Some(encoder.finish()));

    let buf_slice = output_buf.slice(..);
    let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
    buf_slice.map_async(wgpu::MapMode::Read, move |res| {
        sender.send(res).unwrap();
    });
    let query_slice = query_staging_buf.slice(..);
    query_slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::MaintainBase::wait());
    if let Some(Ok(())) = receiver.receive().await {
        let data_bytes = &*buf_slice.get_mapped_range();
        let data: &[f32] = bytemuck::cast_slice(data_bytes);
        println!("got back {} elements", data.len());

        // YOLO, it's probably been mapped by now
        let query_bytes = &*query_slice.get_mapped_range();
        let query: &[u64] = bytemuck::cast_slice(query_bytes);
        let ts_period = queue.get_timestamp_period();
        println!(
            "query took {:.3}ms",
            (query[1] - query[0]) as f32 * ts_period * 1e-3
        );
    }

    Ok(())
}

fn main() {
    color_eyre::install().unwrap();
    pollster::block_on(run()).unwrap()
}
