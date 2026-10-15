//! Generic WASI ABI request/reply order. This does not grant any TraceFix task step.
use hibana::{
    g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::protocol;
pub const GUEST: u8 = 0;
pub const ENV: u8 = 1;
pub type Exit = g::Msg<240, i32>;

pub fn choreography() -> impl Projectable {
    let directory = g::seq(
        g::send::<GUEST, ENV, protocol::FdReaddirReqMsg>(),
        g::send::<ENV, GUEST, protocol::FdReaddirRetMsg>(),
    );
    let write_terminal = g::seq(
        g::send::<GUEST, ENV, protocol::FdWriteReqMsg>(),
        g::send::<ENV, GUEST, protocol::FdWriteRetMsg>(),
    );
    let write_object = g::seq(
        g::send::<GUEST, ENV, protocol::FdWriteObjectReqMsg>(),
        g::send::<ENV, GUEST, protocol::FdWriteObjectRetMsg>(),
    );
    let read = g::seq(
        g::send::<GUEST, ENV, protocol::FdReadReqMsg>(),
        g::send::<ENV, GUEST, protocol::FdReadRetMsg>(),
    );
    let stat = g::seq(
        g::send::<GUEST, ENV, protocol::FdFdstatGetReqMsg>(),
        g::send::<ENV, GUEST, protocol::FdFdstatGetRetMsg>(),
    );
    let prestat = g::seq(
        g::send::<GUEST, ENV, protocol::FdPrestatGetReqMsg>(),
        g::send::<ENV, GUEST, protocol::FdPrestatGetRetMsg>(),
    );
    let prestat_name = g::seq(
        g::send::<GUEST, ENV, protocol::FdPrestatDirNameReqMsg>(),
        g::send::<ENV, GUEST, protocol::FdPrestatDirNameRetMsg>(),
    );
    let file_stat = g::seq(
        g::send::<GUEST, ENV, protocol::FdFilestatGetReqMsg>(),
        g::send::<ENV, GUEST, protocol::FdFilestatGetRetMsg>(),
    );
    let close = g::seq(
        g::send::<GUEST, ENV, protocol::FdCloseReqMsg>(),
        g::send::<ENV, GUEST, protocol::FdCloseRetMsg>(),
    );
    let open = g::seq(
        g::send::<GUEST, ENV, protocol::PathOpenReqMsg>(),
        g::send::<ENV, GUEST, protocol::PathOpenRetMsg>(),
    );
    let path_stat = g::seq(
        g::send::<GUEST, ENV, protocol::PathFilestatGetReqMsg>(),
        g::send::<ENV, GUEST, protocol::PathFilestatGetRetMsg>(),
    );
    let memory_grow = g::seq(
        g::send::<GUEST, ENV, protocol::MemoryGrowReqMsg>(),
        g::send::<ENV, GUEST, protocol::MemoryGrowRetMsg>(),
    );
    g::route(
        directory,
        g::route(
            write_terminal,
            g::route(
                write_object,
                g::route(
                    read,
                    g::route(
                        stat,
                        g::route(
                            prestat,
                            g::route(
                                prestat_name,
                                g::route(
                                    file_stat,
                                    g::route(
                                        close,
                                        g::route(
                                            open,
                                            g::route(
                                                path_stat,
                                                g::route(
                                                    memory_grow,
                                                    g::send::<GUEST, ENV, Exit>(),
                                                ),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
    .roll()
}

pub fn programs() -> (RoleProgram<GUEST>, RoleProgram<ENV>) {
    let global = choreography();
    (project(&global), project(&global))
}
