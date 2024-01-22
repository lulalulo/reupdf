use clap::{App, Arg};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use lopdf::{Document, Object};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("reupdf")
        .version("1.0")
        .about("Merges PDF files")
        .arg(Arg::new("INPUT")
            .help("Input PDF files")
            .required(true)
            .multiple_values(true) // Allows multiple input files
            .min_values(1)         // At least one input file is required
            .takes_value(true))
        .arg(Arg::new("OUTPUT")
            .help("Output PDF file")
            .required(true)
            .takes_value(true))
        .get_matches();

    // Extract input files and the output file from the matches
    let input_files: Vec<_> = matches.values_of("INPUT").unwrap().collect();
    let output_file = matches.value_of("OUTPUT").unwrap();

    // Process the PDF merging...
    let mut output_doc = Document::with_version("1.5");
    for file in input_files {
        let reader = BufReader::new(File::open(file)?);
        let doc = Document::load_from(reader)?;
        merge_documents(&mut output_doc, doc)?;
    }

    output_doc.save(Path::new(output_file))?;

    Ok(())
}

fn merge_documents(output: &mut Document, mut other: Document) -> Result<(), Box<dyn std::error::Error>> {
    let mut max_id = output.max_id as usize + 1;
    other.renumber_objects_with(max_id as u32);

    for (&object_id, object) in &other.objects {
        output.objects.insert(object_id, object.clone());
        max_id = max_id.max(object_id.0 as usize);
    }

    output.max_id = max_id as u32;

    let other_pages = other.get_pages();

    for (_, &page_id) in &other_pages {
        let new_object_id = (max_id as u32, 0);
        max_id += 1;
        // Use page_id directly as it is already an ObjectId
        output.objects.insert(new_object_id, Object::Reference(page_id));
    }

    let mut kids = Vec::new();
    for (_, &page_id) in &output.get_pages() {
        kids.push(Object::Reference(page_id));
    }

    if let Ok(root) = output.trailer.get_mut(b"Root").and_then(|o| o.as_dict_mut()) {
        if let Ok(pages) = root.get_mut(b"Pages").and_then(|o| o.as_dict_mut()) {
            pages.set("Count", Object::Integer(kids.len() as i64));
            pages.set("Kids", Object::Array(kids));
        }
    }

    Ok(())
}

