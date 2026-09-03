// build.rs — embutido automaticamente pelo Cargo antes de compilar
//
// Incorpora o manifesto Windows (elevação UAC) diretamente no .exe
// usando o crate `winres` — sem dependência externa obrigatória:
// se winres não estiver disponível, o build continua normalmente.

fn main() {
    // Só roda no alvo Windows
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        // Tenta embutir o manifesto; falha silenciosa se winres não instalado
        embed_manifest();
    }
}

#[cfg(target_os = "windows")]
fn embed_manifest() {
    // Para usar: adicione ao Cargo.toml:
    //   [build-dependencies]
    //   winres = "0.1"
    //
    // Descomente o bloco abaixo após adicionar a dependência:
    //
    // let mut res = winres::WindowsResource::new();
    // res.set_manifest_file("LimpezaTemp.manifest");
    // res.set("FileDescription", "Limpeza de Temporários e Cache");
    // res.set("ProductName", "LimpezaTemp");
    // res.set("LegalCopyright", "2024");
    // res.compile().expect("Falha ao compilar recursos Windows");
}

#[cfg(not(target_os = "windows"))]
fn embed_manifest() {
    // Nada a fazer em outros sistemas
}
