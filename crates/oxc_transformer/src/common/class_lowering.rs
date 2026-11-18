//! Per-class ownership of class lowering.
//!
//! Several transforms need to know whether class-properties lowers a class, and why. The shape
//! that decision depends on - decorators and auto-accessors - is removed or rewritten early in the
//! transform, so the decision is recorded once per class while the class is still untouched.
//!
//! Transforms look up the decision by class scope after the class is entered.

use oxc_ast::ast::Class;
use oxc_syntax::scope::ScopeId;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::{
    decorator::DecoratorMode,
    utils::decorated_class::{class_body_has_auto_accessors, class_lowered_for_decorators},
};

use super::class_transform::ClassTransform;

/// Which transform owns a class.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClassLoweringMode {
    /// Class fields stay native; auto-accessors and static blocks may still be lowered.
    Native,
    /// Class-properties lowers this class because the target requires class field lowering.
    LowerAll,
    /// The class keeps its decorators and auto-accessors for a downstream transform.
    Preserve,
}

/// Ownership of one class, decided from its untransformed AST.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ClassLowering {
    mode: ClassLoweringMode,
    lowered_for_decorators: bool,
    has_auto_accessor: bool,
}

impl ClassLowering {
    fn compute(class_transform: ClassTransform, class: &Class<'_>) -> Self {
        let lowered_for_decorators = class_lowered_for_decorators(class);
        let has_auto_accessor = class_body_has_auto_accessors(&class.body);
        // Classes with decorators or auto-accessors are preserved for a downstream transform.
        let mode = if lowered_for_decorators || has_auto_accessor {
            ClassLoweringMode::Preserve
        } else {
            match class_transform {
                ClassTransform::ClassProperties { .. } => ClassLoweringMode::LowerAll,
                ClassTransform::None | ClassTransform::StaticBlocks => ClassLoweringMode::Native,
            }
        };
        Self { mode, lowered_for_decorators, has_auto_accessor }
    }

    pub fn mode(self) -> ClassLoweringMode {
        self.mode
    }

    /// Whether the class or one of its elements is decorated.
    pub fn lowered_for_decorators(self) -> bool {
        self.lowered_for_decorators
    }

    pub fn has_auto_accessor(self) -> bool {
        self.has_auto_accessor
    }

    /// Whether this decision depends on syntax which is removed or rewritten later.
    fn depends_on_rewritten_syntax(self) -> bool {
        self.lowered_for_decorators || self.has_auto_accessor
    }
}

/// Number of records kept without allocating.
const INLINE_RECORDS: usize = 8;

/// Recorded [`ClassLowering`] for every class whose decision is still needed later, keyed by class
/// scope.
pub struct ClassLoweringCache {
    class_transform: ClassTransform,
    decorator_mode: DecoratorMode,
    /// Decorated classes are rare, so the common cases are held inline without allocating.
    entries: SmallVec<[(ScopeId, ClassLowering); INLINE_RECORDS]>,
    spilled: FxHashMap<ScopeId, ClassLowering>,
}

impl ClassLoweringCache {
    pub fn new(class_transform: ClassTransform, decorator_mode: DecoratorMode) -> Self {
        Self {
            class_transform,
            decorator_mode,
            entries: SmallVec::new(),
            spilled: FxHashMap::default(),
        }
    }

    /// Record the lowering of a class before its decorators or auto-accessors are rewritten.
    pub fn record(&mut self, class: &Class<'_>) {
        // Legacy decorators run their own lowering, so options alone determine class ownership.
        if self.decorator_mode.is_legacy() {
            return;
        }
        let lowering = ClassLowering::compute(self.class_transform, class);
        if lowering.depends_on_rewritten_syntax() {
            self.insert(class.scope_id(), lowering);
        }
    }

    /// Look up the lowering of an entered class.
    pub fn get(&self, scope_id: ScopeId) -> ClassLowering {
        self.recorded(scope_id).unwrap_or_else(|| self.unrecorded())
    }

    fn recorded(&self, scope_id: ScopeId) -> Option<ClassLowering> {
        if self.spilled.is_empty() {
            self.entries.iter().find(|(id, _)| *id == scope_id).map(|(_, lowering)| *lowering)
        } else {
            self.spilled.get(&scope_id).copied()
        }
    }

    fn insert(&mut self, scope_id: ScopeId, lowering: ClassLowering) {
        if self.spilled.is_empty() {
            if self.entries.len() < INLINE_RECORDS {
                self.entries.push((scope_id, lowering));
                return;
            }
            self.spilled.extend(self.entries.drain(..));
        }
        self.spilled.insert(scope_id, lowering);
    }

    /// The options-based decision for legacy mode and for classes without runtime decorators or
    /// auto-accessors.
    fn unrecorded(&self) -> ClassLowering {
        let mode = match self.class_transform {
            ClassTransform::ClassProperties { .. } => ClassLoweringMode::LowerAll,
            ClassTransform::None | ClassTransform::StaticBlocks => ClassLoweringMode::Native,
        };
        ClassLowering { mode, lowered_for_decorators: false, has_auto_accessor: false }
    }
}

impl Default for ClassLoweringCache {
    fn default() -> Self {
        // Matches the default options: no class lowering, standard decorators preserved.
        Self::new(ClassTransform::default(), DecoratorMode::Preserve)
    }
}
