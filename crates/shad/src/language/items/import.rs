use crate::compilation::node::{choice, sequence, NodeConfig, Repeated};
use crate::compilation::validation::ValidationContext;
use crate::compilation::FILE_EXT;
use crate::language::keywords::{DotSymbol, ImportKeyword, SemicolonSymbol, TildeSymbol};
use crate::language::patterns::Ident;
use crate::ValidationError;
use std::path::{Path, PathBuf};
use std::rc::Rc;

sequence!(
    struct ImportItem {
        import: ImportKeyword,
        #[force_error(true)]
        path_prefix: Repeated<ImportPathPrefix, 0, { usize::MAX }>,
        path_suffix: Ident,
        semicolon: SemicolonSymbol,
    }
);

impl NodeConfig for ImportItem {
    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        let path = self.import_path(ctx.root_path);
        if !path.exists() {
            ctx.errors.push(ValidationError::error(
                ctx,
                self,
                "imported file not found",
                Some(&format!("no file found at `{}`", path.display())),
                &[],
            ));
        }
    }
}

impl ImportItem {
    pub(crate) fn import_path(&self, root_path: &Path) -> PathBuf {
        let segments = self
            .path_prefix
            .iter()
            .map(|prefix| prefix.segment.clone())
            .chain([Rc::new(ImportPathSegment::Ident(self.path_suffix.clone()))])
            .collect::<Vec<_>>();
        let mut path = match &*segments[0] {
            ImportPathSegment::Parent(_) => self.path.clone(),
            ImportPathSegment::Ident(_) => root_path.to_path_buf(),
        };
        for segment in segments {
            match &*segment {
                ImportPathSegment::Parent(_) => path = path.parent().unwrap_or(&path).to_path_buf(),
                ImportPathSegment::Ident(ident) => path.push(&ident.slice),
            }
        }
        path.set_extension(FILE_EXT);
        path
    }
}

sequence!(
    #[allow(unused_mut)]
    struct ImportPathPrefix {
        segment: ImportPathSegment,
        dot: DotSymbol,
    }
);

impl NodeConfig for ImportPathPrefix {}

choice!(
    enum ImportPathSegment {
        Parent(TildeSymbol),
        Ident(Ident),
    }
);

impl NodeConfig for ImportPathSegment {}
