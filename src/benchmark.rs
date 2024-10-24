use criterion::Criterion;
use miette::{Context, IntoDiagnostic, Result};
use openvino::{CompiledModel, InferRequest};

fn load_model(path: impl AsRef<str>) -> Result<CompiledModel> {
    let mut core = openvino::Core::new().into_diagnostic()?;
    let model = core
        .read_model_from_file(path.as_ref(), path.as_ref())
        .into_diagnostic()?;
    core.compile_model(&model, openvino::DeviceType::CPU)
        .into_diagnostic()
}

/// Create a dummy infer request with zero-filled input tensors.
#[inline]
fn create_dummy_infer_request(model: &mut CompiledModel) -> Result<InferRequest> {
    let mut request = model.create_infer_request().into_diagnostic()?;
    let num_inputs = model.get_input_size().into_diagnostic()?;

    for i in 0..num_inputs {
        let input_node = model.get_input_by_index(i).into_diagnostic()?;
        let shape = input_node.get_shape().into_diagnostic()?;
        let element_type = input_node.get_element_type().into_diagnostic()?;

        let tensor = openvino::Tensor::new(element_type, &shape).into_diagnostic()?;

        request
            .set_input_tensor_by_index(i, &tensor)
            .into_diagnostic()?;
    }

    Ok(request)
}

/// Run a benchmark on an ONNX model.
pub fn run_benchmark_onnx(path: impl AsRef<str>, c: &mut Criterion) -> Result<()> {
    let mut model = load_model(path).context("failed to load model")?;

    c.bench_function("onnx inference", |b| {
        b.iter(|| {
            let mut request =
                create_dummy_infer_request(&mut model).expect("failed to create inference request");

            request.infer().expect("failed to complete inference");
        });
    });

    Ok(())
}