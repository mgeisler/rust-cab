extern crate cab;
extern crate chrono;
extern crate lipsum;

use chrono::NaiveDate;
use std::io::{Cursor, Read, Write};

// ========================================================================= //

#[test]
fn cabinet_with_one_small_uncompressed_file() {
    let original = lipsum::lipsum(500);
    let datetime = NaiveDate::from_ymd(2063, 4, 5).and_hms(23, 14, 38);

    let mut cab_builder = cab::CabinetBuilder::new();
    {
        let folder_builder = cab_builder
            .add_folder(cab::CompressionType::None);
        let file_builder = folder_builder
            .add_file("lorem_ipsum.txt".to_string());
        file_builder.set_datetime(datetime);
        file_builder.set_is_read_only(true);
        file_builder.set_is_system_file(true);
    }
    let mut cab_writer = cab_builder.build(Cursor::new(Vec::new())).unwrap();
    while let Some(mut file_writer) = cab_writer.next_file().unwrap() {
        file_writer.write_all(original.as_bytes()).unwrap();
    }
    let cab_file = cab_writer.finish().unwrap().into_inner();

    let mut cabinet = cab::Cabinet::new(Cursor::new(cab_file)).unwrap();
    {
        let file_entry = cabinet.get_file_entry("lorem_ipsum.txt").unwrap();
        assert_eq!(file_entry.datetime(), datetime);
        assert!(file_entry.is_read_only());
        assert!(!file_entry.is_hidden());
        assert!(file_entry.is_system_file());
    }
    let mut output = Vec::new();
    let mut file_reader = cabinet.read_file("lorem_ipsum.txt").unwrap();
    file_reader.read_to_end(&mut output).unwrap();
    assert_eq!(String::from_utf8_lossy(&output), original);
}

#[test]
fn cabinet_with_one_small_mszipped_file() {
    let original = lipsum::lipsum(500);

    let mut cab_builder = cab::CabinetBuilder::new();
    cab_builder
        .add_folder(cab::CompressionType::MsZip)
        .add_file("lorem_ipsum.txt".to_string());
    let mut cab_writer = cab_builder.build(Cursor::new(Vec::new())).unwrap();
    while let Some(mut file_writer) = cab_writer.next_file().unwrap() {
        file_writer.write_all(original.as_bytes()).unwrap();
    }
    let cab_file = cab_writer.finish().unwrap().into_inner();

    let mut cabinet = cab::Cabinet::new(Cursor::new(cab_file)).unwrap();
    assert_eq!(cabinet.folder_entries().nth(0).unwrap().compression_type(),
               cab::CompressionType::MsZip);
    let mut output = Vec::new();
    let mut file_reader = cabinet.read_file("lorem_ipsum.txt").unwrap();
    file_reader.read_to_end(&mut output).unwrap();
    assert_eq!(String::from_utf8_lossy(&output), original);
}

#[test]
fn cabinet_with_one_big_uncompressed_file() {
    let original = lipsum::lipsum(30000);

    let mut cab_builder = cab::CabinetBuilder::new();
    cab_builder
        .add_folder(cab::CompressionType::None)
        .add_file("lorem_ipsum.txt".to_string());
    let mut cab_writer = cab_builder.build(Cursor::new(Vec::new())).unwrap();
    while let Some(mut file_writer) = cab_writer.next_file().unwrap() {
        file_writer.write_all(original.as_bytes()).unwrap();
    }
    let cab_file = cab_writer.finish().unwrap().into_inner();
    assert!(cab_file.len() > original.len());

    let mut cabinet = cab::Cabinet::new(Cursor::new(cab_file)).unwrap();
    {
        let folder = cabinet.folder_entries().nth(0).unwrap();
        assert_eq!(folder.compression_type(), cab::CompressionType::None);
        assert!(folder.num_data_blocks() > 1);
        let file = folder.file_entries().nth(0).unwrap();
        assert_eq!(file.uncompressed_size() as usize, original.len());
    }
    let mut output = Vec::new();
    let mut file_reader = cabinet.read_file("lorem_ipsum.txt").unwrap();
    file_reader.read_to_end(&mut output).unwrap();
    assert_eq!(output.len(), original.len());
    assert_eq!(String::from_utf8_lossy(&output), original);
}

#[test]
fn cabinet_with_one_big_mszipped_file() {
    let original = lipsum::lipsum(30000);

    let mut cab_builder = cab::CabinetBuilder::new();
    cab_builder
        .add_folder(cab::CompressionType::MsZip)
        .add_file("lorem_ipsum.txt".to_string());
    let mut cab_writer = cab_builder.build(Cursor::new(Vec::new())).unwrap();
    while let Some(mut file_writer) = cab_writer.next_file().unwrap() {
        file_writer.write_all(original.as_bytes()).unwrap();
    }
    let cab_file = cab_writer.finish().unwrap().into_inner();
    assert!(cab_file.len() < original.len());

    let mut cabinet = cab::Cabinet::new(Cursor::new(cab_file)).unwrap();
    {
        let folder = cabinet.folder_entries().nth(0).unwrap();
        assert_eq!(folder.compression_type(), cab::CompressionType::MsZip);
        let file = folder.file_entries().nth(0).unwrap();
        assert_eq!(file.uncompressed_size() as usize, original.len());
    }
    let mut output = Vec::new();
    let mut file_reader = cabinet.read_file("lorem_ipsum.txt").unwrap();
    file_reader.read_to_end(&mut output).unwrap();
    assert_eq!(output.len(), original.len());
    assert_eq!(String::from_utf8_lossy(&output), original);
}

// ========================================================================= //