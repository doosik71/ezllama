mod check_cuda_toolkit;
mod check_llama_cpp;

fn main() {
    check_cuda_toolkit::run();
    check_llama_cpp::run();
}
