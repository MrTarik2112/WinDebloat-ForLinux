use windebloat::config::loader::load_config;
use windebloat::os::distro::OSInfo;
use windebloat::utils::error::Result;

fn main() -> Result<()> {
    let config = load_config();
    println!("Config loaded: {:?}", config);
    
    let os = OSInfo::detect();
    println!("OS detected: {:?}", os);
    
    Ok(())
}
