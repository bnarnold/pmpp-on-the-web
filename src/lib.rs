#![allow(dead_code)]

mod ch_03;

use eyre::OptionExt;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter,
};
use wgpu::util::DeviceExt;

#[derive(Debug, PartialEq, Eq)]
struct Matrix<T> {
    entries: Vec<T>,
    width: usize,
    height: usize,
}

impl<T> Matrix<T> {
    fn new(entries: Vec<T>, width: usize, height: usize) -> Self {
        debug_assert_eq!(entries.len(), width * height);
        Self {
            entries,
            width,
            height,
        }
    }
}

impl Matrix<f32> {
    fn mul(&self, rhs: &Self) -> Option<Self> {
        if self.width != rhs.height {
            return None;
        }
        let entry_count = self.height * rhs.width;
        let mut entries = vec![0.0; entry_count];

        for j in 0..rhs.width {
            for (i, row) in self.entries.chunks(self.width).enumerate() {
                entries[i * rhs.width + j] = (0..self.width)
                    .map(|k| rhs.entries[k * rhs.width + j])
                    .zip(row)
                    .map(|(left, right)| left * right)
                    .sum::<f32>();
            }
        }

        Some(Self {
            entries,
            width: rhs.width,
            height: self.height,
        })
    }
}

pub trait ToBytesArray {
    fn to_bytes_array(&self) -> Vec<&[u8]>;
}

impl ToBytesArray for () {
    fn to_bytes_array(&self) -> Vec<&[u8]> {
        Vec::new()
    }
}
macro_rules! impl_to_bytes_array (
    ($(&[$t:ident]),*) => {
        impl<$($t),*> ToBytesArray for ($(&[$t],)*) where $($t: bytemuck::Pod),* {
            #[allow(non_snake_case)]
            fn to_bytes_array(&self) -> Vec<&[u8]> {
                let ($($t),*,) = self;
                vec![$(bytemuck::cast_slice($t)),*]
            }
        }
    };
);

impl_to_bytes_array!(&[T1]);
impl_to_bytes_array!(&[T1], &[T2]);
impl_to_bytes_array!(&[T1], &[T2], &[T3]);
impl_to_bytes_array!(&[T1], &[T2], &[T3], &[T4]);
impl_to_bytes_array!(&[T1], &[T2], &[T3], &[T4], &[T5]);
impl_to_bytes_array!(&[T1], &[T2], &[T3], &[T4], &[T5], &[T6]);
impl_to_bytes_array!(&[T1], &[T2], &[T3], &[T4], &[T5], &[T6], &[T7]);
impl_to_bytes_array!(&[T1], &[T2], &[T3], &[T4], &[T5], &[T6], &[T7], &[T8]);
impl_to_bytes_array!(
    &[T1],
    &[T2],
    &[T3],
    &[T4],
    &[T5],
    &[T6],
    &[T7],
    &[T8],
    &[T9]
);
impl_to_bytes_array!(
    &[T1],
    &[T2],
    &[T3],
    &[T4],
    &[T5],
    &[T6],
    &[T7],
    &[T8],
    &[T9],
    &[T10]
);
impl_to_bytes_array!(
    &[T1],
    &[T2],
    &[T3],
    &[T4],
    &[T5],
    &[T6],
    &[T7],
    &[T8],
    &[T9],
    &[T10],
    &[T11]
);
impl_to_bytes_array!(
    &[T1],
    &[T2],
    &[T3],
    &[T4],
    &[T5],
    &[T6],
    &[T7],
    &[T8],
    &[T9],
    &[T10],
    &[T11],
    &[T12]
);
impl_to_bytes_array!(
    &[T1],
    &[T2],
    &[T3],
    &[T4],
    &[T5],
    &[T6],
    &[T7],
    &[T8],
    &[T9],
    &[T10],
    &[T11],
    &[T12],
    &[T13]
);
impl_to_bytes_array!(
    &[T1],
    &[T2],
    &[T3],
    &[T4],
    &[T5],
    &[T6],
    &[T7],
    &[T8],
    &[T9],
    &[T10],
    &[T11],
    &[T12],
    &[T13],
    &[T14]
);
impl_to_bytes_array!(
    &[T1],
    &[T2],
    &[T3],
    &[T4],
    &[T5],
    &[T6],
    &[T7],
    &[T8],
    &[T9],
    &[T10],
    &[T11],
    &[T12],
    &[T13],
    &[T14],
    &[T15]
);
impl_to_bytes_array!(
    &[T1],
    &[T2],
    &[T3],
    &[T4],
    &[T5],
    &[T6],
    &[T7],
    &[T8],
    &[T9],
    &[T10],
    &[T11],
    &[T12],
    &[T13],
    &[T14],
    &[T15],
    &[T16]
);

#[tracing::instrument(skip_all)]
pub async fn run_shader<R: bytemuck::Pod + Clone>(
    shader: &str,
    inputs: impl ToBytesArray,
    num_workgroups: (u32, u32, u32),
    output_len: usize,
) -> eyre::Result<Vec<R>> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        })
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
            source: wgpu::ShaderSource::Wgsl(shader.into()),
        })
    };
    let input_bufs: Vec<_> = inputs
        .to_bytes_array()
        .into_iter()
        .map(|bytes| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytes,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
            })
        })
        .collect();
    let output_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (output_len * std::mem::size_of::<R>()) as u64,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::MAP_READ,
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

    let input_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &(0..input_bufs.len())
                .map(|i| wgpu::BindGroupLayoutEntry {
                    binding: i as u32,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                })
                .collect::<Vec<_>>(),
        });
    let output_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        bind_group_layouts: &[&input_bind_group_layout, &output_bind_group_layout],
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

    let input_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &input_bind_group_layout,
        entries: &input_bufs
            .iter()
            .enumerate()
            .map(|(i, buf)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: buf,
                    offset: 0,
                    size: None,
                }),
            })
            .collect::<Vec<_>>(),
    });

    let output_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &output_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &output_buf,
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
        cpass.set_bind_group(0, &input_bind_group, &[]);
        cpass.set_bind_group(1, &output_bind_group, &[]);
        cpass.dispatch_workgroups(num_workgroups.0, num_workgroups.1, num_workgroups.2);
    }
    if let Some(query_set) = &query_set {
        encoder.write_timestamp(query_set, 1);
    }
    if let Some(query_set) = &query_set {
        encoder.resolve_query_set(query_set, 0..2, &query_buf, 0);
    }
    encoder.copy_buffer_to_buffer(&query_buf, 0, &query_staging_buf, 0, 16);
    queue.submit(Some(encoder.finish()));

    let buf_slice = output_buf.slice(..);
    let (sender_data, receiver_data) = futures_intrusive::channel::shared::oneshot_channel();
    buf_slice.map_async(wgpu::MapMode::Read, move |res| {
        sender_data.send(res).unwrap();
    });
    let query_slice = query_staging_buf.slice(..);
    let (sender_query, receiver_query) = futures_intrusive::channel::shared::oneshot_channel();
    query_slice.map_async(wgpu::MapMode::Read, move |res| {
        sender_query.send(res).unwrap()
    });

    device.poll(wgpu::MaintainBase::wait());
    if let (Some(Ok(())), _) = futures::join!(receiver_data.receive(), receiver_query.receive()) {
        let data_bytes = &*buf_slice.get_mapped_range();
        let data: &[R] = bytemuck::cast_slice(data_bytes);
        tracing::info!("got back {} elements", data.len());

        let query_bytes = &*query_slice.get_mapped_range();
        let query: &[u64] = bytemuck::cast_slice(query_bytes);
        let ts_period = queue.get_timestamp_period();
        tracing::info!(
            "query took {:.3}ms",
            (query[1] - query[0]) as f32 * ts_period * 1e-3
        );

        return Ok(data.to_owned());
    }

    Err(eyre::eyre!("Could not map buffers"))
}

pub fn init_logging() {
    static ONCE: std::sync::OnceLock<()> = std::sync::OnceLock::new();

    ONCE.get_or_init(|| {
        color_eyre::install().unwrap();
        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(fmt::layer().with_span_events(FmtSpan::CLOSE))
            .init();
    });
}
