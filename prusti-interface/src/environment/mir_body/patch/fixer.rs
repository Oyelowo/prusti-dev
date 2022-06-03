use super::compiler::MirPatch;
use rustc_hash::FxHashSet;
use rustc_middle::mir;

/// A patch generated by drop elaboration may contain dangling blocks. This
/// function fixes that.
pub(super) fn fix_patch<'tcx>(body: &mir::Body<'tcx>, mut patch: MirPatch<'tcx>) -> MirPatch<'tcx> {
    assert_eq!(
        patch.patch_map.len(),
        body.basic_blocks().len() + patch.new_blocks.len()
    );
    let mut reachable_blocks = FxHashSet::default();
    for (bb_index, terminator) in patch.patch_map.iter().enumerate() {
        if bb_index == body.basic_blocks().len() {
            break;
        }
        if let Some(terminator) = terminator {
            for successor in terminator.successors() {
                reachable_blocks.insert(*successor);
            }
        }
    }
    let mut queue: Vec<_> = reachable_blocks.iter().cloned().collect();
    while let Some(bb) = queue.pop() {
        if bb.index() < body.basic_blocks().len() {
            continue;
        }
        let terminator = if let Some(terminator) = &patch.patch_map[bb] {
            terminator
        } else {
            let offset = bb.index() - body.basic_blocks().len();
            &patch.new_blocks[offset].terminator().kind
        };
        for successor in terminator.successors() {
            if reachable_blocks.insert(*successor) {
                queue.push(*successor);
            }
        }
    }
    let mut new_block_offset = patch.new_blocks.len();
    while new_block_offset > 0 {
        assert_eq!(
            patch.patch_map.len(),
            body.basic_blocks().len() + patch.new_blocks.len()
        );
        new_block_offset -= 1;
        let bb: mir::BasicBlock = (body.basic_blocks().len() + new_block_offset).into();
        if !reachable_blocks.contains(&bb) {
            // Remove the block.
            patch.patch_map.raw.remove(bb.index());
            patch.new_blocks.remove(new_block_offset);
            // Shift all other blocks.
            for new_bb_index in new_block_offset..(patch.new_blocks.len()) {
                let new_bb: mir::BasicBlock = (body.basic_blocks().len() + new_bb_index).into();
                let old_bb_index = body.basic_blocks().len() + new_bb_index + 1;
                let old_bb: mir::BasicBlock = old_bb_index.into();
                // Shift all patch_map.
                for terminator in patch.patch_map.iter_mut().flatten() {
                    for successor in terminator.successors_mut() {
                        if *successor == old_bb {
                            *successor = new_bb;
                        }
                    }
                }
                // Shift all new_block.
                for block in &mut patch.new_blocks {
                    for successor in block.terminator_mut().successors_mut() {
                        if *successor == old_bb {
                            *successor = new_bb;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(
        patch.patch_map.len(),
        body.basic_blocks().len() + patch.new_blocks.len()
    );
    patch
}
