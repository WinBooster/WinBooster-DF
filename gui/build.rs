#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
    println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");

    let mut res = winres::WindowsResource::new();
    res.set_icon("../assets\\icon.ico");

    res.set_manifest(
        r#"
    <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
            </requestedPrivileges>
        </security>
    </trustInfo>
    </assembly>
    "#,
    );

    res.compile().unwrap();
}

#[cfg(unix)]
fn main() {}
