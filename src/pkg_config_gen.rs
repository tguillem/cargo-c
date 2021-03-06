#![allow(dead_code)]

use crate::build::CApiConfig;
use crate::install::InstallPaths;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PkgConfig {
    prefix: PathBuf,
    exec_prefix: PathBuf,
    includedir: PathBuf,
    libdir: PathBuf,

    name: String,
    description: String,
    version: String,

    requires: Vec<String>,
    requires_private: Vec<String>,

    libs: Vec<String>,
    libs_private: Vec<String>,

    cflags: Vec<String>,

    conflicts: Vec<String>,
}

impl PkgConfig {
    ///
    /// Build a pkgconfig structure with the following defaults:
    ///
    /// prefix=/usr/local
    /// exec_prefix=${prefix}
    /// includedir=${prefix}/include
    /// libdir=${exec_prefix}/lib
    ///
    /// Name: $name
    /// Description: $description
    /// Version: $version
    /// Cflags: -I${includedir}/$name
    /// Libs: -L${libdir} -l$name
    ///
    pub fn new(name: &str, capi_config: &CApiConfig) -> Self {
        PkgConfig {
            name: capi_config.pkg_config.name.clone(),
            description: capi_config.pkg_config.description.clone(),
            version: capi_config.pkg_config.version.clone(),

            prefix: "/usr/local".into(),
            exec_prefix: "${prefix}".into(),
            includedir: "${prefix}/include".into(),
            libdir: "${exec_prefix}/lib".into(),

            libs: vec![format!("-L{} -l{}", "${libdir}", capi_config.library.name)],
            libs_private: Vec::new(),

            requires: Vec::new(),
            requires_private: Vec::new(),

            cflags: vec![if capi_config.header.subdirectory {
                format!("-I{}/{}", "${includedir}", name)
            } else {
                String::from("-I${includedir}")
            }],

            conflicts: Vec::new(),
        }
    }

    pub(crate) fn from_workspace(
        name: &str,
        install_paths: &InstallPaths,
        args: &structopt::clap::ArgMatches<'_>,
        capi_config: &CApiConfig,
    ) -> Self {
        let mut pc = PkgConfig::new(name, capi_config);

        pc.prefix = install_paths.prefix.clone();
        // TODO: support exec_prefix
        if args.is_present("includedir") {
            pc.includedir = install_paths.includedir.clone();
        }
        if args.is_present("libdir") {
            pc.libdir = install_paths.libdir.clone();
        }
        pc
    }

    pub fn set_description<S: AsRef<str>>(&mut self, descr: S) -> &mut Self {
        self.description = descr.as_ref().to_owned();
        self
    }

    pub fn set_libs<S: AsRef<str>>(&mut self, lib: S) -> &mut Self {
        let lib = lib.as_ref().to_owned();
        self.libs.clear();
        self.libs.push(lib);
        self
    }

    pub fn add_lib<S: AsRef<str>>(&mut self, lib: S) -> &mut Self {
        let lib = lib.as_ref().to_owned();
        self.libs.push(lib);
        self
    }

    pub fn set_libs_private<S: AsRef<str>>(&mut self, lib: S) -> &mut Self {
        let lib = lib.as_ref().to_owned();
        self.libs.clear();
        self.libs.push(lib);
        self
    }

    pub fn add_lib_private<S: AsRef<str>>(&mut self, lib: S) -> &mut Self {
        let lib = lib.as_ref().to_owned();
        self.libs_private.push(lib);
        self
    }

    pub fn set_cflags<S: AsRef<str>>(&mut self, flag: S) -> &mut Self {
        let flag = flag.as_ref().to_owned();
        self.libs.clear();
        self.libs.push(flag);
        self
    }

    pub fn add_cflag<S: AsRef<str>>(&mut self, flag: S) -> &mut Self {
        let flag = flag.as_ref();
        self.libs.push(flag.to_owned());
        self
    }

    pub fn render(&self) -> String {
        let mut base = format!(
            "prefix={}
exec_prefix={}
libdir={}
includedir={}

Name: {}
Description: {}
Version: {}
Libs: {}
Cflags: {}",
            self.prefix.display(),
            self.exec_prefix.display(),
            self.libdir.display(),
            self.includedir.display(),
            self.name,
            self.description,
            self.version,
            self.libs.join(" "),
            self.cflags.join(" "),
        );

        if !self.libs_private.is_empty() {
            base.push_str(&format!(
                "
Libs.private: {}",
                self.libs_private.join(" "),
            ));
        }

        if !self.requires.is_empty() {
            base.push_str(&format!(
                "
Requires: {}",
                self.requires.join(", ")
            ));
        }
        /*
        Requires: libavresample >= 4.0.0, libavutil >= 56.8.0
        Requires.private:
        Conflicts:
        Libs.private:

                ).to_owned()
        */

        base.push_str("\n");

        base
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use semver::Version;

    #[test]
    fn simple() {
        let mut pkg = PkgConfig::new(
            "foo",
            &CApiConfig {
                header: crate::build::HeaderCApiConfig {
                    name: "foo".into(),
                    subdirectory: true,
                    generation: true,
                },
                pkg_config: crate::build::PkgConfigCApiConfig {
                    name: "foo".into(),
                    description: "".into(),
                    version: "0.1".into(),
                },
                library: crate::build::LibraryCApiConfig {
                    name: "foo".into(),
                    version: Version::parse("0.1.0").unwrap(),
                },
            },
        );
        pkg.add_lib("-lbar").add_cflag("-DFOO");

        println!("{:?}\n{}", pkg, pkg.render());
    }
}
