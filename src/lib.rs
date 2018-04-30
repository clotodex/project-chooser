/* dir to search in (default ~/projects)
 *
 * search query
 * if exact match => cd to there
 * if multiple matches => show menu to select
 * if no match => show message
 *
 * basename mode on/off
 * option to count exact basename match in non basename mode
 *
 * use a cache file
 *
 * make highly parallel with channels and a threadpool
 *
 * a match is:
 * - folder contains .git or .project
 *
 * optimize search:
 * - ignore .git folders
 * - ignore folders marked as .noproject
 * - optionally src/ directories etc
 * - [advanced] ignore all files/folders of a .projectignore
 *
 * OR do not decend in folders with .project => rather have .projectgroup
 *
 * future:
 * - let query be piped in
 * - config for adding ignored directories
 * - max depth for descending
 */

pub mod search;
pub mod walker;
pub mod cache;
