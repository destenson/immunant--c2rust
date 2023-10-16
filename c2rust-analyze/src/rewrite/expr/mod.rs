use crate::context::{AnalysisCtxt, Assignment};
use crate::pointee_type::PointeeTypes;
use crate::pointer_id::PointerTable;
use crate::rewrite::Rewrite;
use rustc_hir::BodyId;
use rustc_middle::mir::Body;
use rustc_span::Span;

mod convert;
mod distribute;
mod mir_op;
mod unlower;

// Helpers used by the shim builder.
pub use self::convert::convert_cast_rewrite;
pub use self::mir_op::CastBuilder;

pub fn gen_expr_rewrites<'tcx>(
    acx: &AnalysisCtxt<'_, 'tcx>,
    asn: &Assignment,
    pointee_types: PointerTable<PointeeTypes<'tcx>>,
    mir: &Body<'tcx>,
    hir_body_id: BodyId,
) -> Vec<(Span, Rewrite)> {
    let mir_rewrites = mir_op::gen_mir_rewrites(acx, asn, pointee_types, mir);
    let unlower_map = unlower::unlower(acx.tcx(), mir, hir_body_id);
    let rewrites_by_expr = distribute::distribute(acx.tcx(), unlower_map, mir_rewrites);
    let hir_rewrites = convert::convert_rewrites(acx.tcx(), hir_body_id, rewrites_by_expr);
    hir_rewrites
}
