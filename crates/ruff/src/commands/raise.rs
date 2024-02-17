use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::{stderr, stdout, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Result;
use colored::Colorize;
use itertools::Itertools;
use log::{error, warn};
use rayon::iter::Either::{Left, Right};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rustc_hash::FxHashSet;
use thiserror::Error;
use tracing::debug;

use ruff_diagnostics::SourceMap;
use ruff_linter::fs;
use ruff_linter::logging::{DisplayParseError, LogLevel};
use ruff_linter::registry::Rule;
use ruff_linter::rules::flake8_quotes::settings::Quote;
use ruff_linter::source_kind::{SourceError, SourceKind};
use ruff_linter::warn_user_once;
use ruff_python_ast::{PySourceType, SourceType};
use ruff_python_formatter::{format_module_source, format_range, FormatModuleError, QuoteStyle};
use ruff_source_file::LineIndex;
use ruff_text_size::{TextLen, TextRange, TextSize};
use ruff_workspace::resolver::{match_exclusion, python_files_in_path, ResolvedFile, Resolver};
use ruff_workspace::FormatterSettings;

use crate::args::{CliOverrides, RaiseArguments, FormatRange};
use crate::cache::{Cache, FileCacheKey, PackageCacheMap, PackageCaches};
use crate::panic::{catch_unwind, PanicError};
use crate::resolve::resolve;
use crate::{resolve_default_files, ExitStatus};


/// Find exceptions on a set of files, and return the exit status.
pub(crate) fn raise(
    cli: RaiseArguments,
    overrides: &CliOverrides,
    log_level: LogLevel,
) -> Result<ExitStatus> {
    let pyproject_config = resolve(
        false,
        None,
        overrides,
        None,
    )?;
    let files = resolve_default_files(cli.files, false);
    let (paths, resolver) = python_files_in_path(&files, &pyproject_config, overrides)?;

    if paths.is_empty() {
        warn_user_once!("No Python files found under the given path(s)");
        return Ok(ExitStatus::Success);
    }
    // Discover the package root for each Python file.
    let package_roots = resolver.package_roots(
        &paths
            .iter()
            .flatten()
            .map(ResolvedFile::path)
            .collect::<Vec<_>>(),
    );

    let caches = if cli.no_cache {
        None
    } else {
        // `--no-cache` doesn't respect code changes, and so is often confusing during
        // development.
        #[cfg(debug_assertions)]
        crate::warn_user!("Detected debug build without --no-cache.");

        Some(PackageCacheMap::init(&package_roots, &resolver))
    };

    let start = Instant::now();
    // let (results, mut errors): (Vec<_>, Vec<_>) = paths
    //     .par_iter()
    //     .filter_map(|entry| {
    //         match entry {
    //             Ok(resolved_file) => {
    //                 let path = resolved_file.path();
    //                 let settings = resolver.resolve(path);

    //                 let source_type = match settings.formatter.extension.get(path) {
    //                     None => match SourceType::from(path) {
    //                         SourceType::Python(source_type) => source_type,
    //                         SourceType::Toml(_) => {
    //                             // Ignore any non-Python files.
    //                             return None;
    //                         }
    //                     },
    //                     Some(language) => PySourceType::from(language),
    //                 };

    //                 // Ignore files that are excluded from formatting
    //                 if (settings.file_resolver.force_exclude || !resolved_file.is_root())
    //                     && match_exclusion(
    //                         path,
    //                         resolved_file.file_name(),
    //                         &settings.formatter.exclude,
    //                     )
    //                 {
    //                     return None;
    //                 }

    //                 let package = path
    //                     .parent()
    //                     .and_then(|parent| package_roots.get(parent).copied())
    //                     .flatten();
    //                 let cache_root = package.unwrap_or_else(|| path.parent().unwrap_or(path));
    //                 let cache = caches.get(cache_root);

    //                 Some(
    //                     match catch_unwind(|| {
    //                         raise_path(
    //                             path,
    //                             source_type,
    //                             cache,
    //                         )
    //                     }) {
    //                         Ok(inner) => inner.map(|result| FormatPathResult {
    //                             path: resolved_file.path().to_path_buf(),
    //                             result,
    //                         }),
    //                         Err(error) => Err(FormatCommandError::Panic(
    //                             Some(resolved_file.path().to_path_buf()),
    //                             error,
    //                         )),
    //                     },
    //                 )
    //             }
    //             Err(err) => Some(Err(FormatCommandError::Ignore(err.clone()))),
    //         }
    //     })
    //     .partition_map(|result| match result {
    //         Ok(diagnostic) => Left(diagnostic),
    //         Err(err) => Right(err),
    //     });
    let duration = start.elapsed();

    // debug!(
    //     "Formatted {} files in {:.2?}",
    //     results.len() + errors.len(),
    //     duration
    // );

    caches.persist()?;

    // Report on any errors.
    // errors.sort_unstable_by(|a, b| a.path().cmp(&b.path()));

    // for error in &errors {
    //     error!("{error}");
    // }

    // let results = FormatResults::new(results.as_slice(), mode);
    // match mode {
    //     FormatMode::Write => {}
    //     FormatMode::Check => {
    //         results.write_changed(&mut stdout().lock())?;
    //     }
    //     FormatMode::Diff => {
    //         results.write_diff(&mut stdout().lock())?;
    //     }
    // }

    // Report on the formatting changes.
    // if log_level >= LogLevel::Default {
    //     if mode.is_diff() {
    //         // Allow piping the diff to e.g. a file by writing the summary to stderr
    //         results.write_summary(&mut stderr().lock())?;
    //     } else {
    //         results.write_summary(&mut stdout().lock())?;
    //     }
    // }
    Ok(ExitStatus::Success)
}

/// Format the file at the given [`Path`].
// #[tracing::instrument(level="debug", skip_all, fields(path = %path.display()))]
// pub(crate) fn raise_path(
//     path: &Path,
//     source_type: PySourceType,
//     cache: Option<&Cache>,
// ) -> Result<FormatResult, FormatCommandError> {
//     if let Some(cache) = cache {
//         let relative_path = cache
//             .relative_path(path)
//             .expect("wrong package cache for file");

//         if let Ok(cache_key) = FileCacheKey::from_path(path) {
//             if cache.is_formatted(relative_path, &cache_key) {
//                 return Ok(FormatResult::Unchanged);
//             }
//         }
//     }

//     // Extract the sources from the file.
//     let unformatted = match SourceKind::from_path(path, source_type) {
//         Ok(Some(source_kind)) => source_kind,
//         // Non-Python Jupyter notebook.
//         Ok(None) => return Ok(FormatResult::Skipped),
//         Err(err) => {
//             return Err(FormatCommandError::Read(Some(path.to_path_buf()), err));
//         }
//     };


//     // Format the source.
//     let format_result = match raise_source(&unformatted, source_type, Some(path), settings, range)?
//     {
//         FormattedSource::Formatted(formatted) => match mode {
//             FormatMode::Write => {
//                 let mut writer = File::create(path).map_err(|err| {
//                     FormatCommandError::Write(Some(path.to_path_buf()), err.into())
//                 })?;
//                 formatted
//                     .write(&mut writer)
//                     .map_err(|err| FormatCommandError::Write(Some(path.to_path_buf()), err))?;

//                 if let Some(cache) = cache {
//                     if let Ok(cache_key) = FileCacheKey::from_path(path) {
//                         let relative_path = cache
//                             .relative_path(path)
//                             .expect("wrong package cache for file");
//                         cache.set_formatted(relative_path.to_path_buf(), &cache_key);
//                     }
//                 }

//                 FormatResult::Formatted
//             }
//             FormatMode::Check => FormatResult::Formatted,
//             FormatMode::Diff => FormatResult::Diff {
//                 unformatted,
//                 formatted,
//             },
//         },
//         FormattedSource::Unchanged => {
//             if let Some(cache) = cache {
//                 if let Ok(cache_key) = FileCacheKey::from_path(path) {
//                     let relative_path = cache
//                         .relative_path(path)
//                         .expect("wrong package cache for file");
//                     cache.set_formatted(relative_path.to_path_buf(), &cache_key);
//                 }
//             }

//             FormatResult::Unchanged
//         }
//     };

//     Ok(format_result)
// }

#[derive(Debug)]
pub(crate) enum FormattedSource {
    /// The source was formatted, and the [`SourceKind`] contains the transformed source code.
    Formatted(SourceKind),
    /// The source was unchanged.
    Unchanged,
}

// impl From<FormattedSource> for FormatResult {
//     fn from(value: FormattedSource) -> Self {
//         match value {
//             FormattedSource::Formatted(_) => FormatResult::Formatted,
//             FormattedSource::Unchanged => FormatResult::Unchanged,
//         }
//     }
// }

/// Format a [`SourceKind`], returning the transformed [`SourceKind`], or `None` if the source was
/// unchanged.
pub(crate) fn raise_source(
    source_kind: &SourceKind,
    source_type: PySourceType,
    path: Option<&Path>,
    settings: &FormatterSettings,
    range: Option<FormatRange>,
) -> Result<FormattedSource, RaiseCommandError> {
    match &source_kind {
        SourceKind::Python(unformatted) => {
            let options = settings.to_format_options(source_type, unformatted);

            let formatted = if let Some(range) = range {
                let line_index = LineIndex::from_source_text(unformatted);
                let byte_range = range.to_text_range(unformatted, &line_index);
                format_range(unformatted, byte_range, options).map(|formatted_range| {
                    let mut formatted = unformatted.to_string();
                    formatted.replace_range(
                        std::ops::Range::<usize>::from(formatted_range.source_range()),
                        formatted_range.as_code(),
                    );

                    formatted
                })
            } else {
                // Using `Printed::into_code` requires adding `ruff_formatter` as a direct dependency, and I suspect that Rust can optimize the closure away regardless.
                #[allow(clippy::redundant_closure_for_method_calls)]
                format_module_source(unformatted, options).map(|formatted| formatted.into_code())
            };

            let formatted = formatted.map_err(|err| {
                if let FormatModuleError::ParseError(err) = err {
                    DisplayParseError::from_source_kind(
                        err,
                        path.map(Path::to_path_buf),
                        source_kind,
                    )
                    .into()
                } else {
                    RaiseCommandError::Format(path.map(Path::to_path_buf), err)
                }
            })?;

            if formatted.len() == unformatted.len() && formatted == *unformatted {
                Ok(FormattedSource::Unchanged)
            } else {
                Ok(FormattedSource::Formatted(SourceKind::Python(formatted)))
            }
        }
        SourceKind::IpyNotebook(notebook) => {
            if !notebook.is_python_notebook() {
                return Ok(FormattedSource::Unchanged);
            }

            if range.is_some() {
                return Err(RaiseCommandError::RangeFormatNotebook(
                    path.map(Path::to_path_buf),
                ));
            }

            let options = settings.to_format_options(source_type, notebook.source_code());

            let mut output: Option<String> = None;
            let mut last: Option<TextSize> = None;
            let mut source_map = SourceMap::default();

            // Format each cell individually.
            for (start, end) in notebook.cell_offsets().iter().tuple_windows::<(_, _)>() {
                let range = TextRange::new(*start, *end);
                let unformatted = &notebook.source_code()[range];

                // Format the cell.
                let formatted =
                    format_module_source(unformatted, options.clone()).map_err(|err| {
                        if let FormatModuleError::ParseError(err) = err {
                            DisplayParseError::from_source_kind(
                                err,
                                path.map(Path::to_path_buf),
                                source_kind,
                            )
                            .into()
                        } else {
                            RaiseCommandError::Format(path.map(Path::to_path_buf), err)
                        }
                    })?;

                // If the cell is unchanged, skip it.
                let formatted = formatted.as_code();
                if formatted.len() == unformatted.len() && formatted == unformatted {
                    continue;
                }

                // If this is the first newly-formatted cell, initialize the output.
                let output = output
                    .get_or_insert_with(|| String::with_capacity(notebook.source_code().len()));

                // Add all contents from `last` to the current cell.
                let slice = &notebook.source_code()
                    [TextRange::new(last.unwrap_or_default(), range.start())];
                output.push_str(slice);

                // Add the start source marker for the cell.
                source_map.push_marker(*start, output.text_len());

                // Add the cell itself.
                output.push_str(formatted);

                // Add the end source marker for the added cell.
                source_map.push_marker(*end, output.text_len());

                // Track that the cell was formatted.
                last = Some(*end);
            }

            // If the file was unchanged, return `None`.
            let (Some(mut output), Some(last)) = (output, last) else {
                return Ok(FormattedSource::Unchanged);
            };

            // Add the remaining content.
            let slice = &notebook.source_code()[usize::from(last)..];
            output.push_str(slice);

            // Update the notebook.
            let mut formatted = notebook.clone();
            formatted.update(&source_map, output);

            Ok(FormattedSource::Formatted(SourceKind::IpyNotebook(
                formatted,
            )))
        }
    }
}

/// The result of an individual formatting operation.
#[derive(Debug, Clone, is_macro::Is)]
pub(crate) enum RaiseResult {
    Raises {
        raises: HashMap<String, String>
    },
}

/// The coupling of a [`RaiseResult`] with the path of the file that was analyzed.
#[derive(Debug)]
struct RaisePathResult {
    path: PathBuf,
    result: RaiseResult,
}

/// The results of formatting a set of files
// #[derive(Debug)]
// struct FormatResults<'a> {
//     /// The individual formatting results.
//     results: &'a [FormatPathResult],
//     /// The format mode that was used.
//     mode: FormatMode,
// }

// impl<'a> FormatResults<'a> {
//     fn new(results: &'a [FormatPathResult], mode: FormatMode) -> Self {
//         Self { results, mode }
//     }

//     /// Returns `true` if any of the files require formatting.
//     fn any_formatted(&self) -> bool {
//         self.results.iter().any(|result| match result.result {
//             FormatResult::Formatted | FormatResult::Diff { .. } => true,
//             FormatResult::Unchanged | FormatResult::Skipped => false,
//         })
//     }

//     /// Write a diff of the formatting changes to the given writer.
//     fn write_diff(&self, f: &mut impl Write) -> io::Result<()> {
//         for (path, unformatted, formatted) in self
//             .results
//             .iter()
//             .filter_map(|result| {
//                 if let FormatResult::Diff {
//                     unformatted,
//                     formatted,
//                 } = &result.result
//                 {
//                     Some((result.path.as_path(), unformatted, formatted))
//                 } else {
//                     None
//                 }
//             })
//             .sorted_unstable_by_key(|(path, _, _)| *path)
//         {
//             unformatted.diff(formatted, Some(path), f)?;
//         }

//         Ok(())
//     }

//     /// Write a list of the files that would be changed to the given writer.
//     fn write_changed(&self, f: &mut impl Write) -> io::Result<()> {
//         for path in self
//             .results
//             .iter()
//             .filter_map(|result| {
//                 if result.result.is_formatted() {
//                     Some(result.path.as_path())
//                 } else {
//                     None
//                 }
//             })
//             .sorted_unstable()
//         {
//             writeln!(f, "Would reformat: {}", fs::relativize_path(path).bold())?;
//         }

//         Ok(())
//     }

//     /// Write a summary of the formatting results to the given writer.
//     fn write_summary(&self, f: &mut impl Write) -> io::Result<()> {
//         // Compute the number of changed and unchanged files.
//         let mut changed = 0u32;
//         let mut unchanged = 0u32;
//         for result in self.results {
//             match &result.result {
//                 FormatResult::Formatted => {
//                     changed += 1;
//                 }
//                 FormatResult::Unchanged => unchanged += 1,
//                 FormatResult::Diff { .. } => {
//                     changed += 1;
//                 }
//                 FormatResult::Skipped => {}
//             }
//         }

//         // Write out a summary of the formatting results.
//         if changed > 0 && unchanged > 0 {
//             writeln!(
//                 f,
//                 "{} file{} {}, {} file{} {}",
//                 changed,
//                 if changed == 1 { "" } else { "s" },
//                 match self.mode {
//                     FormatMode::Write => "reformatted",
//                     FormatMode::Check | FormatMode::Diff => "would be reformatted",
//                 },
//                 unchanged,
//                 if unchanged == 1 { "" } else { "s" },
//                 match self.mode {
//                     FormatMode::Write => "left unchanged",
//                     FormatMode::Check | FormatMode::Diff => "already formatted",
//                 },
//             )
//         } else if changed > 0 {
//             writeln!(
//                 f,
//                 "{} file{} {}",
//                 changed,
//                 if changed == 1 { "" } else { "s" },
//                 match self.mode {
//                     FormatMode::Write => "reformatted",
//                     FormatMode::Check | FormatMode::Diff => "would be reformatted",
//                 }
//             )
//         } else if unchanged > 0 {
//             writeln!(
//                 f,
//                 "{} file{} {}",
//                 unchanged,
//                 if unchanged == 1 { "" } else { "s" },
//                 match self.mode {
//                     FormatMode::Write => "left unchanged",
//                     FormatMode::Check | FormatMode::Diff => "already formatted",
//                 },
//             )
//         } else {
//             Ok(())
//         }
//     }
// }

/// An error that can occur while formatting a set of files.
#[derive(Error, Debug)]
pub(crate) enum RaiseCommandError {
    Ignore(#[from] ignore::Error),
    Parse(#[from] DisplayParseError),
    Panic(Option<PathBuf>, PanicError),
    Read(Option<PathBuf>, SourceError),
    Format(Option<PathBuf>, FormatModuleError),
    Write(Option<PathBuf>, SourceError),
    Diff(Option<PathBuf>, io::Error),
    RangeFormatNotebook(Option<PathBuf>),
}

impl RaiseCommandError {
    fn path(&self) -> Option<&Path> {
        match self {
            Self::Ignore(err) => {
                if let ignore::Error::WithPath { path, .. } = err {
                    Some(path.as_path())
                } else {
                    None
                }
            }
            Self::Parse(err) => err.path(),
            Self::Panic(path, _)
            | Self::Read(path, _)
            | Self::Format(path, _)
            | Self::Write(path, _)
            | Self::Diff(path, _)
            | Self::RangeFormatNotebook(path) => path.as_deref(),
        }
    }
}

impl Display for RaiseCommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ignore(err) => {
                if let ignore::Error::WithPath { path, .. } = err {
                    write!(
                        f,
                        "{}{}{} {}",
                        "Failed to format ".bold(),
                        fs::relativize_path(path).bold(),
                        ":".bold(),
                        err.io_error()
                            .map_or_else(|| err.to_string(), std::string::ToString::to_string)
                    )
                } else {
                    write!(
                        f,
                        "{header} {error}",
                        header = "Encountered error:".bold(),
                        error = err
                            .io_error()
                            .map_or_else(|| err.to_string(), std::string::ToString::to_string)
                    )
                }
            }
            Self::Parse(err) => {
                write!(f, "{err}")
            }
            Self::Read(path, err) => {
                if let Some(path) = path {
                    write!(
                        f,
                        "{}{}{} {err}",
                        "Failed to read ".bold(),
                        fs::relativize_path(path).bold(),
                        ":".bold()
                    )
                } else {
                    write!(f, "{header} {err}", header = "Failed to read:".bold())
                }
            }
            Self::Write(path, err) => {
                if let Some(path) = path {
                    write!(
                        f,
                        "{}{}{} {err}",
                        "Failed to write ".bold(),
                        fs::relativize_path(path).bold(),
                        ":".bold()
                    )
                } else {
                    write!(f, "{header} {err}", header = "Failed to write:".bold())
                }
            }
            Self::Format(path, err) => {
                if let Some(path) = path {
                    write!(
                        f,
                        "{}{}{} {err}",
                        "Failed to raise ".bold(),
                        fs::relativize_path(path).bold(),
                        ":".bold()
                    )
                } else {
                    write!(f, "{header} {err}", header = "Failed to raise:".bold())
                }
            }
            Self::Diff(path, err) => {
                if let Some(path) = path {
                    write!(
                        f,
                        "{}{}{} {err}",
                        "Failed to generate diff for ".bold(),
                        fs::relativize_path(path).bold(),
                        ":".bold()
                    )
                } else {
                    write!(
                        f,
                        "{header} {err}",
                        header = "Failed to generate diff:".bold(),
                    )
                }
            }
            Self::RangeFormatNotebook(path) => {
                if let Some(path) = path {
                    write!(
                        f,
                        "{header}{path}{colon} Range formatting isn't supported for notebooks.",
                        header = "Failed to format ".bold(),
                        path = fs::relativize_path(path).bold(),
                        colon = ":".bold()
                    )
                } else {
                    write!(
                        f,
                        "{header} Range formatting isn't supported for notebooks",
                        header = "Failed to format:".bold()
                    )
                }
            }
            Self::Panic(path, err) => {
                let message = r"This indicates a bug in Ruff. If you could open an issue at:

    https://github.com/astral-sh/ruff/issues/new?title=%5BFormatter%20panic%5D

...with the relevant file contents, the `pyproject.toml` settings, and the following stack trace, we'd be very appreciative!
";
                if let Some(path) = path {
                    write!(
                        f,
                        "{}{}{} {message}\n{err}",
                        "Panicked while formatting ".bold(),
                        fs::relativize_path(path).bold(),
                        ":".bold()
                    )
                } else {
                    write!(
                        f,
                        "{} {message}\n{err}",
                        "Panicked while formatting.".bold()
                    )
                }
            }
        }
    }
}
