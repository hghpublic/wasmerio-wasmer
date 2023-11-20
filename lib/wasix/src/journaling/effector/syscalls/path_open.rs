use super::*;

impl JournalEffector {
    #[allow(clippy::too_many_arguments)]
    pub fn save_path_open(
        ctx: &mut FunctionEnvMut<'_, WasiEnv>,
        fd: Fd,
        dirfd: Fd,
        dirflags: LookupFlags,
        path: String,
        o_flags: Oflags,
        fs_rights_base: Rights,
        fs_rights_inheriting: Rights,
        fs_flags: Fdflags,
    ) -> anyhow::Result<()> {
        Self::save_event(
            ctx,
            JournalEntry::OpenFileDescriptor {
                fd,
                dirfd,
                dirflags,
                path: path.into(),
                o_flags,
                fs_rights_base,
                fs_rights_inheriting,
                fs_flags,
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn apply_path_open(
        ctx: &mut FunctionEnvMut<'_, WasiEnv>,
        fd: Fd,
        dirfd: Fd,
        dirflags: LookupFlags,
        path: &str,
        o_flags: Oflags,
        fs_rights_base: Rights,
        fs_rights_inheriting: Rights,
        fs_flags: Fdflags,
    ) -> anyhow::Result<()> {
        let res = crate::syscalls::path_open_internal(
            ctx,
            dirfd,
            dirflags,
            path,
            o_flags,
            fs_rights_base,
            fs_rights_inheriting,
            fs_flags,
        );
        let ret_fd = match res? {
            Ok(fd) => fd,
            Err(err) => {
                bail!(
                    "journal restore error: failed to open descriptor (fd={}, path={}) - {}",
                    fd,
                    path,
                    err
                );
            }
        };

        let ret = crate::syscalls::fd_renumber_internal(ctx, ret_fd, fd);
        if ret != Errno::Success {
            bail!(
                "journal restore error: failed renumber file descriptor after open (from={}, to={}) - {}",
                ret_fd,
                fd,
                ret
            );
        }

        Ok(())
    }
}
