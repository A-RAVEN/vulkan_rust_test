use std::fs;
use std::io;
use std::io::Read;

use shaderc::CompilationArtifact;
use shaderc::CompileOptions;

pub enum ShaderType
{
    Vertex,
    Fragment,
    Compute,
}

pub fn load_shader_src(path: &str) -> io::Result<String>{
    let file = fs::File::open(path)?;
    let mut buf_reader = io::BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn load_and_compile_shader_src(path: &str, shader_type: ShaderType) -> CompilationArtifact{

    let shader_source = load_shader_src(&path).unwrap();
    let mut compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.add_macro_definition("ENTRY_POINT", Some("main"));

    let mut shaderc_shader_kind = shaderc::ShaderKind::Vertex;
    match shader_type
    {
        ShaderType::Vertex => {
            options.add_macro_definition("SHADER_FREQUENCY_VERTEX", Some("1"));
            shaderc_shader_kind = shaderc::ShaderKind::Vertex;
        },
        ShaderType::Fragment => {
            options.add_macro_definition("SHADER_FREQUENCY_FRAGMENT", Some("1"));
            shaderc_shader_kind = shaderc::ShaderKind::Fragment;

        },
        ShaderType::Compute => {
            options.add_macro_definition("SHADER_FREQUENCY_COMPUTE", Some("1"));
            shaderc_shader_kind = shaderc::ShaderKind::Compute;
        },
    }
    let binary_result = compiler.compile_into_spirv(&shader_source
        , shaderc_shader_kind
        , "vertex.glsl"
        , "main"
        , Some(&options)).unwrap();

    binary_result
}