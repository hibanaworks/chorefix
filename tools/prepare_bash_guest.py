#!/usr/bin/env python3
"""Prepare the pinned Bash source with an explicit WASI std::fs boundary.

This prepares a guest dependency only. It never executes the shell natively,
changes a TraceFix baseline, or substitutes another Wasm engine or filesystem.
"""

from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "vendor/rust-bash"
REVISION = "68d6dbaf7ae059e29cfe1570a421b0e710ec8572"


def replace(path, before, after):
    text = path.read_text()
    if text.count(before) != 1:
        raise RuntimeError(f"pinned source context changed: {path}")
    path.write_text(text.replace(before, after))


def prepare():
    if not SOURCE.exists():
        subprocess.run(["git", "clone", "https://github.com/shantanugoel/rust-bash.git", str(SOURCE)], check=True)
        subprocess.run(["git", "-C", str(SOURCE), "checkout", "--detach", REVISION], check=True)
    revision = subprocess.check_output(["git", "-C", str(SOURCE), "rev-parse", "HEAD"], text=True).strip()
    if revision != REVISION:
        raise RuntimeError("Bash dependency is not at the recorded revision")
    diff = subprocess.check_output(["git", "-C", str(SOURCE), "diff", "--binary", "--unified=0"])
    recorded = ROOT / "guests/bash/wasip1.patch"
    if diff:
        if not recorded.exists() or diff != recorded.read_bytes():
            raise RuntimeError("Bash dependency has unrecorded edits")
        return

    replace(SOURCE / "src/platform.rs", '#[cfg(target_arch = "wasm32")]',
            '#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]')
    replace(SOURCE / "src/platform.rs", '#[cfg(not(target_arch = "wasm32"))]',
            '#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]')

    # WASI has capability rights rather than POSIX chmod. Do not claim that
    # changing Unix mode bits succeeded, or maintain an extra permission table.
    for name in ["readwrite", "overlay"]:
        path = SOURCE / f"src/vfs/{name}.rs"
        replace(path, "use std::os::unix::fs::PermissionsExt;",
                '#[cfg(not(target_os = "wasi"))]\nuse std::os::unix::fs::PermissionsExt;')
        replace(path, "        mode: meta.permissions().mode(),",
                '        #[cfg(not(target_os = "wasi"))]\n        mode: meta.permissions().mode(),\n'
                '        #[cfg(target_os = "wasi")]\n'
                '        mode: if meta.is_dir() { 0o040777 } else { 0o100666 },')
    path = SOURCE / "src/vfs/readwrite.rs"
    replace(path,
            '        let resolved = self.resolve_follow(path)?;\n'
            '        let perms = std::fs::Permissions::from_mode(mode);\n'
            '        std::fs::set_permissions(&resolved, perms).map_err(|e| map_io_error(e, path))',
            '        #[cfg(target_os = "wasi")]\n        {\n'
            '            let _ = (path, mode);\n'
            '            Err(VfsError::IoError("POSIX chmod is not a WASI Preview 1 operation".into()))\n'
            '        }\n        #[cfg(not(target_os = "wasi"))]\n        {\n'
            '            let resolved = self.resolve_follow(path)?;\n'
            '            let perms = std::fs::Permissions::from_mode(mode);\n'
            '            std::fs::set_permissions(&resolved, perms).map_err(|e| map_io_error(e, path))\n'
            '        }')
    replace(path, '        std::os::unix::fs::symlink(&actual_target, &resolved_link)\n'
            '            .map_err(|e| map_io_error(e, link))',
            '        #[cfg(target_os = "wasi")]\n        {\n'
            '            use std::os::fd::AsRawFd;\n'
            '            let parent = resolved_link.parent().ok_or_else(|| VfsError::InvalidPath(link.display().to_string()))?;\n'
            '            let directory = std::fs::File::open(parent).map_err(|e| map_io_error(e, link))?;\n'
            '            let target = actual_target.to_str().ok_or_else(|| VfsError::InvalidPath(target.display().to_string()))?;\n'
            '            let name = resolved_link.file_name().and_then(|name| name.to_str()).ok_or_else(|| VfsError::InvalidPath(link.display().to_string()))?;\n'
            '            // SAFETY: both paths are valid UTF-8 and the directory owns this fd.\n'
            '            unsafe { wasi::path_symlink(target, directory.as_raw_fd() as u32, name) }\n'
            '                .map_err(|e| map_io_error(std::io::Error::from_raw_os_error(e.raw() as i32), link))\n'
            '        }\n        #[cfg(not(target_os = "wasi"))]\n        {\n'
            '            std::os::unix::fs::symlink(&actual_target, &resolved_link).map_err(|e| map_io_error(e, link))\n'
            '        }')
    replace(SOURCE / "Cargo.toml", '[dev-dependencies]',
            '[target.\'cfg(target_os = "wasi")\'.dependencies]\nwasi = "0.11.1"\n\n[dev-dependencies]')

    # A supplied filesystem is already owned and provisioned by the caller.
    # Do not create fake /bin command files or /dev nodes in the WASI filesystem.
    # The interpreter's registered commands remain available without those files.
    path = SOURCE / "src/api.rs"
    replace(path, '        let fs: Arc<dyn VirtualFs> = self.fs.unwrap_or_else(|| Arc::new(InMemoryFs::new()));\n', '')
    replace(path, '        fs.mkdir_p(Path::new(&cwd))?;\n\n'
            '        for (path, content) in &self.files {\n'
            '            let p = Path::new(path);\n'
            '            if let Some(parent) = p.parent()\n'
            '                && parent != Path::new("/")\n'
            '            {\n                fs.mkdir_p(parent)?;\n            }\n'
            '            fs.write_file(p, content)?;\n        }\n', '')
    replace(path, '        setup_default_filesystem(fs.as_ref(), &env_map, &commands)?;',
            '        let fs: Arc<dyn VirtualFs> = match self.fs {\n'
            '            Some(fs) => fs,\n'
            '            None => {\n'
            '                let fs = Arc::new(InMemoryFs::new());\n'
            '                setup_default_filesystem(fs.as_ref(), &env_map, &commands)?;\n'
            '                fs\n            }\n        };\n'
            '        fs.mkdir_p(Path::new(&cwd))?;\n'
            '        for (path, content) in &self.files {\n'
            '            let p = Path::new(path);\n'
            '            if let Some(parent) = p.parent()\n'
            '                && parent != Path::new("/")\n'
            '            {\n                fs.mkdir_p(parent)?;\n            }\n'
            '            fs.write_file(p, content)?;\n        }')
    diff = subprocess.check_output(["git", "-C", str(SOURCE), "diff", "--binary", "--unified=0"])
    if recorded.exists() and recorded.read_bytes() != diff:
        raise RuntimeError("prepared guest dependency differs from its recorded patch")
    recorded.write_bytes(diff)


if __name__ == "__main__":
    prepare()
