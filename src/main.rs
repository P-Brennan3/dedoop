use std::collections::{HashMap, VecDeque};
use std::io;
use std::error::Error;
use std::fs::{self,File};
use std::io::{BufReader, Read};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use sha2::{Sha256, Digest};

fn main() -> Result<(), Box<dyn Error>> {
  let input_pattern = std::env::args().nth(1);

  // 1. Get the target directory, defaulting to the OS's root dir
  #[cfg(windows)]
  let root = "C:\\";

  #[cfg(not(windows))]
  let root = String::from("/");

  let dir = match input_pattern {
    Some(str) => str,
    None => root
  };


  // we need a map of the file size to the files that have it
  let mut size_files_map: HashMap<u64, Vec<String>> = HashMap::new();

  let mut dirs_to_visit: VecDeque<String> = VecDeque::from([dir]);

  // continue looping while we have directories to visit
  while !dirs_to_visit.is_empty() {
    let dir = dirs_to_visit.pop_front().expect("We should always have a value here");
    println!("Searching {dir}...");
    let files = fs::read_dir(dir)?;
    // 2. for each file:
    for entry in files {
        let entry = entry?;
        let path = entry.path();
        let path_str = path.to_str().expect("Error").to_string();

        // add the directory to the stack
        if path.is_dir() {
          dirs_to_visit.push_front(path_str.clone());
        }

        // insert this file size into the map
        let file = File::open(&path)?;
        let size = file.metadata()?.size();
        size_files_map.entry(size).or_insert_with(Vec::new).push(path_str);
    }

  }

  for (size, files) in &size_files_map {
    let files_count = files.len();
    let mut sha_files_map: HashMap<String, Vec<String>> = HashMap::new();
    if files_count > 1 {
      println!("Comparing {files_count} file(s) of size {size}");
        
      // we need a map of the file hash to the files that have it
        for file in files {
            match hash_file(&file) {
            // if we have a hash returned and not an error
            // 1. get the entry in the HashMap for this hash
            // 2. if it doesnt exist, make the value a new emtpy Vector
            // 3. either way push into the vector the path for the file
            Ok(hash) => {
              sha_files_map.entry(hash).or_insert_with(Vec::new).push(String::from(file))
            },
            Err(_) => ()
          } 
        }
    }
    for (_, files) in &sha_files_map {
      if files.len() > 1 {
        println!("These files are a duplicate of eachother {:?}", files)
      }
    }     
  }

  Ok(())
}

fn hash_file(path: &String) -> io::Result<String> {

  // open the file 
  let file = File::open(path)?;
  let size = file.metadata()?.size();
  println!("{size}");
  // return with Err if the file is empty
  if file.metadata()?.len() == 0 {
    return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "file is empty"
    ));
  }

  // create buffer reader 
  let mut reader = BufReader::new(file);

  // initialize the hasher which will incremently hash the file
  let mut hasher = Sha256::new();

  // create a 8KB buffer full of zeros for reading in the file in chunks
  let mut buffer = [0; 8192];

  // keep reading chunk until we reach the end of the file
  loop {
    // read the bytes into the buffer and get the number of bytes read
    let num_bytes_read = reader.read(&mut buffer)?;

    // break when we reach the end of the file (read in zero bytes)
    if num_bytes_read == 0 {
      break
    }

    // update the hash with the chunk of the file we just read in
    // only using the part of the newly filled 8KB max buffer we just filled
    hasher.update(&buffer[..num_bytes_read])
  }
  Ok(format!("{:x}", hasher.finalize()))
}