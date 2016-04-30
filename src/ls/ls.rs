#![crate_name = "uu_ls"]

// This file is part of the uutils coreutils package.
//
// (c) Jordy Dickinson <jordy.dickinson@gmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
//

extern crate getopts;
extern crate winapi;
extern crate kernel32;
extern crate libc;
#[macro_use]
extern crate uucore;

// use kernel32;
use getopts::Options;
use std::fs;
use std::fs::File;
use std::os::windows::fs::MetadataExt;
// use std::fs::{ReadDir, DirEntry, FileType, Metadata};
use std::cmp::Ordering;
// use std::io::{ErrorKind, Result, Write};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
// use libc::c_void;


// static NAME: &'static str = "ls";
// static VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub fn uumain(args: Vec<String>) -> i32 {
    // let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("A", "", "List all entries except for '.' and '..'. ");
    opts.optflag("a",
                 "",
                 "Include directory entries whose names begin with a dot(‘.’).");
    opts.optflag("C",
                 "",
                 "Force multi-column output; this is the default when output is to a terminal.");
    opts.optflag("c",
                 "",
                 "Use time when file status was last changed, instead of time of last \
                  modification of the file for sort-ing ( −t )o rp rinting ( −l ).");
    opts.optflag("d",
                 "",
                 "Directories are listed as plain files (not searched recursively) and symbolic \
                  links in the argumentlist are not indirected through.");
    opts.optflag("F", "", "Display a slash(‘/’ )immediately after each pathname that is a directory,anasterisk (‘∗’) aftereach that is executable, an at sign(‘@’ )after each symbolic link, a percent sign(‘%’ )after eachwhiteout, an equal sign(‘=’ )after each socket, and a vertical bar(‘|’ )after each that is aFIFO.");
    opts.optflag("f", "", "Output is not sorted");
    opts.optflag("h",
                 "",
                 "Modifies the−sand−loptions, causing the sizes to be reported in bytes \
                  displayed in a humanreadable format. Overrides −k.");
    opts.optflag("i",
                 "",
                 "Foreach file, print the file’sfile serial number (inode number).");
    opts.optflag("k",
                 "",
                 "Modifies the−soption, causing the sizes to be reported in kilobytes.The \
                  rightmost of the−kand−h flags overrides the previous flag. See also −h.");
    opts.optflag("l",
                 "",
                 "(The lowercase letter “ell”). List in long format.");
    opts.optflag("n",
                 "",
                 "The same as−l,except that the owner and group IDs are displayed numerically \
                  rather than con-ve rting to a o wner or group name.");
    opts.optflag("q", "", "Force printing of non-printable characters in file names as the character ‘?’; this is the default whenoutput is to a terminal.");
    opts.optflag("R", "", "Recursively list subdirectories encountered.");
    opts.optflag("r",
                 "",
                 "Reverse the order of the sort to get reverse lexicographical order or the \
                  smallest or oldest entriesfirst.");
    opts.optflag("S", "", "Sort by size, largest file first.");
    opts.optflag("s",
                 "",
                 "Display the number of file system blocks actually used by each file, in units \
                  of 512 bytes orBLOCKSIZE (see ENVIRONMENT)w here partial units are rounded up \
                  to the next integer value.If the output is to a terminal, a total sum for all \
                  the file sizes is output on a line before the listing.");
    opts.optflag("t",
                 "",
                 "Sort by time modified (most recently modified first) before sorting the \
                  operands by lexicographicalorder .");
    opts.optflag("u",
                 "",
                 "Use time of last access, instead of last modification of the file for sorting \
                  (−t)orprinting (−l).");
    opts.optflag("w",
                 "",
                 "Force rawprinting of non-printable characters. This is the default when output \
                  is not to a terminal.");
    opts.optflag("x",
                 "",
                 "Force rawprinting of non-printable characters. This is the default when output \
                  is not to a terminal.");
    opts.optflag("1",
                 "",
                 "(The numeric digit “one”).Force output to be one entry per line. This is the \
                  default when outputis not to a terminal.");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    // let sort_fctn = if matches.opt_present("S") {
    //     size_cmp
    // }
    // else {
    //     name_cmp
    // };

    if (&matches).free.is_empty() {
        print!(".\n", );
        traverse(&String::from("."), &matches);
    } else {
        for path in &matches.free {
            print!("{}\n", path);
            traverse(path, &matches);
        }
    }
    0
}

fn traverse(path: &String, matches: &getopts::Matches) {
    let mut stack = vec![PathBuf::from(path)];
    while !stack.is_empty() {
        let mut entries: Vec<_> = fs::read_dir(Path::new(stack.pop().unwrap().as_path()))
                                      .unwrap()
                                      .map(|res| res.unwrap().path())
                                      .collect();
        if matches.opt_present("R") {
            stack.extend(entries.drain(..).filter(|e| e.is_dir()));
        }

        print(matches, &mut entries);
    }
}

fn print(matches: &getopts::Matches, entries: &mut Vec<PathBuf>) {
    if !matches.opt_present("f") {
        if !matches.opt_present("r") {
            if matches.opt_present("S") {
                entries.sort_by(size_cmp);
            } else {
                entries.sort_by(name_cmp);
            }
        } else {
            if matches.opt_present("S") {
                entries.sort_by(re_size_cmp)
            } else {
                entries.sort_by(re_name_cmp);
            }
        }
    }
    if matches.opt_present("l") {
        long_print(matches, entries);
    } else {
        print!("{:?}", entries);
    }
}
// all items inside one folder and just one level
fn long_print(matches: &getopts::Matches, entries: &mut Vec<PathBuf>) {
    let col_width = compute_width(entries, matches);
    print!("Long form{:?}\n", entries);
}

#[cfg(target_os = "windows")]
fn compute_width(entries: &mut Vec<PathBuf>,
                 matches: &getopts::Matches)
                 -> HashMap<&'static str, usize> {
    // let max_ino = entries.iter().map(|x| x.metadata().unwrap().dev()).max();
    // print!("{:?}", max_ino);
    if matches.opt_present("i") {
        print!("Currently Windows' version ls doesn't support inode number.");
    }
    let max_size = entries.iter()
                          .cloned()
                          .filter(|x| !x.is_dir())
                          .fold(0u64, |x, y| std::cmp::max(x, y.metadata().unwrap().len()));
    let max_name_size = entries.iter().cloned().fold(0usize, |x, y| {
        std::cmp::max(x, y.file_name().unwrap().to_str().unwrap().len())
    });

    // print!("Max c_time: {:?}\n", max_c_time);
    for entry in entries {
        if entry.file_name().unwrap().to_str().unwrap() == "." {
            continue;
        }

        print!("{:?}\n", entry.file_name().unwrap());
    } 
    let mut col_width = HashMap::new();
    col_width.insert("btotal", 0usize);
    col_width.insert("stotal", 0);
    col_width.insert("entries", 0);
    col_width.insert("maxlen", 0);
    col_width.insert("s_block", 0);
    col_width.insert("s_flags", 0);
    col_width.insert("s_group", 0);
    col_width.insert("s_inode", 0);
    col_width.insert("s_nlink", 0);
    col_width.insert("s_size", format!("{}", max_size).len());
    col_width.insert("s_user", 0);
    col_width.insert("s_major", 0);
    col_width.insert("s_minor", 0);
    return col_width.clone();
}

#[cfg(target_os = "linux")]
fn compute_width(entries: &mut Vec<PathBuf>,
                 matches: &getopts::Matches)
                 -> HashMap<&'static str, usize> {
    // let max_ino = entries.iter().map(|x| x.metadata().unwrap().dev()).max();
    // print!("{:?}", max_ino);
    if matches.opt_present("i") {
        print!("Currently Windows' version ls doesn't support inode number.");
    }
    let max_size = entries.iter()
                          .cloned()
                          .filter(|x| !x.is_dir())
                          .fold(0u64, |x, y| std::cmp::max(x, y.metadata().unwrap().len()));
    let max_name_size = entries.iter().cloned().fold(0usize, |x, y| {
        std::cmp::max(x, y.file_name().unwrap().to_str().unwrap().len())
    });

    let max_time = if matches.opt_present("u") {
        entries.iter().cloned().filter(|x| !x.is_dir()).fold(0u64, |x, y| {
            std::cmp::max(x, y.metadata().unwrap().last_access_time())
        });
    } else {
        // status change or file modification
        entries.iter().cloned().fold(0u64, |x, y| {
            std::cmp::max(x, y.metadata().unwrap().last_write_time())
        });
    };

    // print!("Max c_time: {:?}\n", max_c_time);
    for entry in entries {
        if entry.file_name().unwrap().to_str().unwrap() == "." {
            continue;
        }

        print!("{:?}\n", entry.file_name().unwrap());
    }
    let mut col_width = HashMap::new();
    col_width.insert("btotal", 0usize);
    col_width.insert("stotal", 0);
    col_width.insert("entries", 0);
    col_width.insert("maxlen", 0);
    col_width.insert("s_block", 0);
    col_width.insert("s_flags", 0);
    col_width.insert("s_group", 0);
    col_width.insert("s_inode", 0);
    col_width.insert("s_nlink", 0);
    col_width.insert("s_size", 0);
    col_width.insert("s_user", 0);
    col_width.insert("s_major", 0);
    col_width.insert("s_minor", 0);
    return col_width.clone();
}

fn name_cmp(a: &PathBuf, b: &PathBuf) -> Ordering {
    a.cmp(b)
}

fn re_name_cmp(a: &PathBuf, b: &PathBuf) -> Ordering {
    b.cmp(a)
}

fn size_cmp(a: &PathBuf, b: &PathBuf) -> Ordering {
    a.metadata().unwrap().len().cmp(&b.metadata().unwrap().len())
}

fn re_size_cmp(a: &PathBuf, b: &PathBuf) -> Ordering {
    b.metadata().unwrap().len().cmp(&a.metadata().unwrap().len())
}
