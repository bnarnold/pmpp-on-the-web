#![allow(dead_code)]

use eyre::Context;

use crate::{run_shader, Matrix};

async fn matmul_by_row(left: &Matrix<f32>, right: &Matrix<f32>) -> eyre::Result<Matrix<f32>> {
    if left.width != right.height {
        return Err(eyre::eyre!("dimension mismatch"));
    }
    let entries = run_shader(
        include_str!("./mmul_by_row.wgsl"),
        (
            left.entries.as_slice(),
            [left.width as u32].as_slice(),
            right.entries.as_slice(),
            [right.width as u32].as_slice(),
        ),
        ((left.height as u32).next_multiple_of(32), 1, 1),
        left.height * right.width,
    )
    .await
    .context("run shader")?;
    Ok(Matrix::new(entries, right.width, left.height))
}
async fn matmul_by_col(left: &Matrix<f32>, right: &Matrix<f32>) -> eyre::Result<Matrix<f32>> {
    if left.width != right.height {
        return Err(eyre::eyre!("dimension mismatch"));
    }
    let entries = run_shader(
        include_str!("./mmul_by_row.wgsl"),
        (
            left.entries.as_slice(),
            [left.width as u32].as_slice(),
            right.entries.as_slice(),
            [right.width as u32].as_slice(),
        ),
        ((right.width as u32).next_multiple_of(32), 1, 1),
        left.height * right.width,
    )
    .await
    .context("run shader")?;
    Ok(Matrix::new(entries, right.width, left.height))
}

#[cfg(test)]
mod tests {
    use crate::init_logging;

    use super::*;

    #[test]
    fn matmul_by_row_square() {
        init_logging();
        let m = Matrix::new(vec![0.0, 1.0, 2.0, 3.0], 2, 2);

        let shader_result = pollster::block_on(matmul_by_row(&m, &m)).unwrap();
        assert_eq!(shader_result, m.mul(&m).unwrap());
    }
    #[test]
    fn matmul_by_col_square() {
        init_logging();
        let m = Matrix::new(vec![0.0, 1.0, 2.0, 3.0], 2, 2);

        let shader_result = pollster::block_on(matmul_by_col(&m, &m)).unwrap();
        assert_eq!(shader_result, m.mul(&m).unwrap());
    }
}
