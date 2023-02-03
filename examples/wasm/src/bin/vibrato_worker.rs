use gloo_worker::Registrable;
use vibrato_wasm::VibratoWorker;

fn main() {
    VibratoWorker::registrar().register();
}
