//Notepad CLI

use std::fs;
use std::io;
use std::path::Path;

// The folder where all notes are stored
// This folder is created automatically when the app first runs
const NOTES_DIR: &str = "notes";

// -------------------------------------------------------
// HELPER — reads a non-empty line from the user
// -------------------------------------------------------
fn read_input(prompt: &str) -> String {
    loop {
        println!("{}", prompt);
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read");
        let trimmed = input.trim().to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
        println!("Input cannot be empty. Please try again.");
    }
}

// -------------------------------------------------------
// HELPER — builds the full file path from a note name
// e.g. "shopping" becomes "notes/shopping.txt"
// -------------------------------------------------------
fn note_path(name: &str) -> String {
    format!("{}/{}.txt", NOTES_DIR, name)
}

// -------------------------------------------------------
// HELPER — lists all notes and returns their names
// Returns an empty Vec if no notes exist
// -------------------------------------------------------
fn get_all_notes() -> Vec<String> {
    let entries = match fs::read_dir(NOTES_DIR) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut names: Vec<String> = Vec::new();

    for entry in entries {
        if let Ok(entry) = entry {
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();

            if let Some(note_name) = name_str.strip_suffix(".txt") {
                names.push(note_name.to_string());
            }
        }
    }
    names.sort();
    names
}

// -------------------------------------------------------
// HELPER — prints the numbered list of all notes
// Returns how many notes exist (0 means none)
// -------------------------------------------------------
fn display_note_list() -> usize {
    let notes = get_all_notes();

    if notes.is_empty() {
        println!("  No notes found. Create one first!");
        return 0;
    }

    println!("\n  Your notes:");
    for (i, name) in notes.iter().enumerate() {
        println!("  {}. {}", i + 1, name);
    }

    notes.len()
}

// -------------------------------------------------------
// HELPER — user picks a note from the list by number
// Returns Some(note_name) or None if cancelled/no notes
// -------------------------------------------------------
fn pick_note(action: &str) -> Option<String> {
    let notes = get_all_notes();

    if notes.is_empty() {
        println!("  No notes exist yet.");
        return None;
    }

    println!("\n  Choose a note to {}:", action);
    for (i, name) in notes.iter().enumerate() {
        println!("  {}. {}", i + 1, name);
    }

    loop {
        println!("  Enter number (or 0 to cancel):");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed");

        match input.trim().parse::<usize>() {
            Ok(0) => return None,
            Ok(n) if n <= notes.len() => return Some(notes[n - 1].clone()),
            _ => println!("  Please enter a number between 1 and {}.", notes.len()),
        }
    }
}
//Helper-reads multi line content inputs
fn read_multiline(prompt: &str) -> String {
    println!("{}", prompt);
    println!("(Type END on a new line itself to finish):");
    let mut content = String::new();
    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line).expect("Failed");
        let line = line.trim_end();
        if line.to_uppercase() == "END" {
            break;
        }
        content.push_str(line);
        content.push('\n');
    }
    content
}

// -------------------------------------------------------
// FEATURE 1 — Create a new note
// -------------------------------------------------------
fn create_note() {
    println!("\n--- Create New Note ---");

    let name = read_input("Enter note name (e.g. shopping, diary, ideas):");
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        println!("  Invalid name: no slashes or '..' allowed.");
        return;
    }
    let name = name.replace(" ", "_");
    let path = note_path(&name);
    if Path::new(&path).exists() {
        println!("  A note named '{}' already exists.", name);
        println!("  Use 'Edit' to overwrite or edit the content. Returning to menu.");
        return;
    }
    let content = read_multiline("Enter the content:");
    match fs::write(&path, &content) {
        Ok(_) => println!("\n  Note '{}' saved successfully!", name),
        Err(e) => println!("\n  Failed to save note: {}", e),
    }
}

// -------------------------------------------------------
// FEATURE 2 — List all notes
// -------------------------------------------------------
fn list_notes() {
    println!("\n--- All Notes ---");
    let count = display_note_list();
    if count > 0 {
        println!("\n  Total: {} note(s)", count);
    }
}

// -------------------------------------------------------
// FEATURE 3 — Read a note
// -------------------------------------------------------
fn read_note() {
    println!("\n--- Read Note ---");

    let name = match pick_note("read") {
        Some(n) => n,
        None => return,
    };

    let path = note_path(&name);

    match fs::read_to_string(&path) {
        Ok(content) => {
            println!("\n{}", "=".repeat(30));
            println!("  NOTE: {}", name.replace("_", " "));
            println!("{}", "=".repeat(30));
            println!("{}", content);
            println!("{}\n", "=".repeat(30));
        }
        Err(e) => println!("  Failed to read note: {}", e),
    }
}

// -------------------------------------------------------
// FEATURE 4 — Edit/Overwrite a note completely
// -------------------------------------------------------
fn edit_note() {
    println!("\n--- Edit Note ---");

    let name = match pick_note("edit") {
        Some(n) => n,
        None => return,
    };

    let path = note_path(&name);

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            println!("  Failed to read note: {}", e);
            return;
        }
    };

    println!("Current content:");
    println!("---");
    println!("{}", content.trim());
    println!("---");

    println!("\n  Choose edit mode:");
    println!("  1. Overwrite completely");
    println!(
        "  2. Insert after a specific line (including append — just insert after the last line,or to insert at the top,insert at line 0.)"
    );
    println!("  3. Delete lines");
    println!("  Enter choice:");

    let mut mode_input = String::new();
    io::stdin().read_line(&mut mode_input).expect("Failed");

    match mode_input.trim() {
        // -----------------------------------------------
        // MODE 1 — overwrite
        // -----------------------------------------------
        "1" => {
            let new_content = read_multiline("Enter the new content to insert:");

            match fs::write(&path, &new_content) {
                Ok(_) => println!("\n  Note '{}' overwritten successfully!", name),
                Err(e) => println!("\n  Failed to edit note: {}", e),
            }
        }

        // -----------------------------------------------
        // MODE 2 — insert after a chosen line
        // -----------------------------------------------
        "2" => {
            let content_lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

            if content_lines.is_empty() {
                println!("  This note is empty. Use Overwrite to add content.");
                return;
            }

            println!("\n  Lines in this note:");
            for (i, line) in content_lines.iter().enumerate() {
                println!("  {}. {}", i + 1, line);
            }

            let chosen = loop {
                println!(
                    "\n  Insert after which line? (0 for top,or 1 to {}):",
                    content_lines.len()
                );
                let mut num_input = String::new();
                io::stdin().read_line(&mut num_input).expect("Failed");

                match num_input.trim().parse::<usize>() {
                    Ok(0) => break 0,
                    Ok(n) if n >= 1 && n <= content_lines.len() => break n,
                    _ => println!(
                        "  Please enter a number between 1 and {}.",
                        content_lines.len()
                    ),
                }
            };
            if chosen == 0 {
                println!("\n Inserting at the top.");
            } else {
                println!(
                    "\n  Inserting after line {}: \"{}\"",
                    chosen,
                    content_lines[chosen - 1]
                );
            }
            println!("  Enter new content to insert.");
            println!("  (Type END on a new line by itself to finish):");

            let mut new_lines: Vec<String> = Vec::new();

            loop {
                let mut line = String::new();
                io::stdin().read_line(&mut line).expect("Failed");
                let line = line.trim_end().to_string();
                if line.to_uppercase() == "END" {
                    break;
                }
                new_lines.push(line);
            }

            let mut final_lines: Vec<String> = Vec::new();

            for line in &content_lines[..chosen] {
                final_lines.push(line.clone());
            }
            for line in new_lines {
                final_lines.push(line);
            }
            for line in &content_lines[chosen..] {
                final_lines.push(line.clone());
            }

            let final_content = final_lines.join("\n") + "\n";

            match fs::write(&path, &final_content) {
                Ok(_) => println!("\n  Note '{}' updated successfully!", name),
                Err(e) => println!("\n  Failed to update note: {}", e),
            }
        }
        // -----------------------------------------------
        // MODE 3 — delete lines 
        // -----------------------------------------------
        "3" => {
            let mut content_lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

            if content_lines.is_empty() {
                println!("  No lines to delete.");
                return;
            }

            println!("\n  Lines in this note:");
            for (i, line) in content_lines.iter().enumerate() {
                println!("  {}. {}", i + 1, line);
            }

            loop {
                println!("Enter the line number to delete or q to exit:");
                let mut del_in = String::new();
                io::stdin().read_line(&mut del_in).expect("Failed");
                if del_in.trim() == "q" {
                    break;
                }
                let delete: usize = match del_in.trim().parse() {
                    Ok(n) if n > 0 && n <= content_lines.len() => n,
                    Ok(_) => {
                        println!(
                            "Invalid number,enter a valid number between 1 and {}",
                            content_lines.len()
                        );
                        continue;
                    }
                    Err(_) => {
                        println!(
                            "Invalid input,enter a valid number between 1 and {}",
                            content_lines.len()
                        );
                        continue;
                    }
                };
                content_lines.remove(delete - 1);
                println!("Updated list:");
                for i in 0..content_lines.len() {
                    println!("{}. {}", i + 1, content_lines[i]);
                }
            }
            let final_content = content_lines.join("\n") + "\n";

            match fs::write(&path, &final_content) {
                Ok(_) => println!("\n  Note '{}' updated successfully!", name),
                Err(e) => println!("\n  Failed to update note: {}", e),
            }
        }

        _ => println!("  Invalid choice. Returning to menu."),
    }
}

// -------------------------------------------------------
// FEATURE 5 — Delete a note
// -------------------------------------------------------
fn delete_note() {
    println!("\n--- Delete Note ---");

    let name = match pick_note("delete") {
        Some(n) => n,
        None => return,
    };

    println!(
        "\n  Are you sure you want to delete '{}'? This cannot be undone.",
        name
    );
    println!("  Type YES to confirm, anything else to cancel:");

    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm).expect("Failed");

    if confirm.trim().to_uppercase() != "YES" {
        println!("  Deletion cancelled.");
        return;
    }

    let path = note_path(&name);

    match fs::remove_file(&path) {
        Ok(_) => println!("\n  Note '{}' deleted.", name),
        Err(e) => println!("\n  Failed to delete note: {}", e),
    }
}

// -------------------------------------------------------
// FEATURE 6 — Rename a note
// -------------------------------------------------------
fn rename_note() {
    println!("\n--- Rename Note ---");

    let old_name = match pick_note("rename") {
        Some(n) => n,
        None => return,
    };

    let new_name_raw = read_input("Enter new name:");
    if new_name_raw.contains('/') || new_name_raw.contains('\\') || new_name_raw.contains("..") {
        println!("  Invalid name: no slashes or '..' allowed.");
        return;
    }
    let new_name = new_name_raw.replace(" ", "_");

    let old_path = note_path(&old_name);
    let new_path = note_path(&new_name);

    if Path::new(&new_path).exists() {
        println!(
            "  A note named '{}' already exists. Choose a different name.",
            new_name
        );
        return;
    }

    match fs::rename(&old_path, &new_path) {
        Ok(_) => println!("\n  Renamed '{}' to '{}' successfully!", old_name, new_name),
        Err(e) => println!("\n  Failed to rename note: {}", e),
    }
}

// -------------------------------------------------------
// FEATURE 7 — Search all notes for a keyword
// -------------------------------------------------------
fn search_notes() {
    println!("\n--- Search Notes ---");

    let keyword = read_input("Enter keyword to search for:");
    let keyword_lower = keyword.to_lowercase();

    let notes = get_all_notes();

    if notes.is_empty() {
        println!("  No notes to search.");
        return;
    }

    let mut found_any = false;

    for name in &notes {
        let path = note_path(name);

        if let Ok(content) = fs::read_to_string(&path) {
            let content_lower = content.to_lowercase();

            if content_lower.contains(&keyword_lower) {
                if !found_any {
                    println!("\n  Results for '{}':", keyword);
                    println!("  ========================================");
                    found_any = true;
                }

                println!("\n  Found in note: {}", name.replace("_", " "));

                for (i, line) in content.lines().enumerate() {
                    if line.to_lowercase().contains(&keyword_lower) {
                        println!("    Line {}: {}", i + 1, line.trim());
                    }
                }
            }
        }
    }

    if !found_any {
        println!("\n  No notes contain '{}'.", keyword);
    } else {
        println!("\n  ========================================");
    }
}

// -------------------------------------------------------
// MAIN MENU
// -------------------------------------------------------
fn main() {
    if let Err(e) = fs::create_dir_all(NOTES_DIR) {
        println!("Could not create notes folder: {}", e);
        return;
    }

    println!("{}", "=".repeat(45));
    println!("              NOTEPAD CLI");
    println!("  Notes are saved in the 'notes/' folder");
    println!("{}\n", "=".repeat(45));

    loop {
        println!("========== MAIN MENU ==========");
        println!("1. Create new note");
        println!("2. List all notes");
        println!("3. Read a note");
        println!("4. Edit a note");
        println!("5. Rename a note");
        println!("6. Delete a note");
        println!("7. Search notes");
        println!("8. Exit");
        println!("{}", "=".repeat(30));
        println!("Choose an option:");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed");

        let choice: usize = match input.trim().parse() {
            Ok(n) => n,
            Err(_) => {
                println!("Please enter a number between 1 and 8.");
                continue;
            }
        };

        match choice {
            1 => create_note(),
            2 => list_notes(),
            3 => read_note(),
            4 => edit_note(),
            5 => rename_note(),
            6 => delete_note(),
            7 => search_notes(),
            8 => {
                println!("\nGoodbye! Your notes are saved in the 'notes/' folder.");
                break;
            }
            _ => println!("Invalid option. Please choose 1 to 8."),
        }
    }
}
