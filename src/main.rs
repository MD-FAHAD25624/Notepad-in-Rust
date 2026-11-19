// Notepad CLI

use std::fs;
use std::io;
use std::path::Path;

const NOTES_DIR: &str = "notes";
const LABELS: [&str; 4] = ["Numbers", "IP Addresses", "Web Addresses", "Alphanumeric Codes"];

// -------------------------------------------------------
// INPUT HELPERS — the ONLY places that touch stdin
// -------------------------------------------------------

// Reads one raw line (trailing newline removed). Exits cleanly on EOF/Ctrl-D
// so the input loops below can never spin forever.
fn read_raw() -> String {
    let mut line = String::new();
    match io::stdin().read_line(&mut line) {
        Ok(0) | Err(_) => {
            println!("\nInput closed. Goodbye!");
            std::process::exit(0);
        }
        Ok(_) => line.trim_end().to_string(),
    }
}

// Reads a non-empty, trimmed line.
fn read_input(prompt: &str) -> String {
    loop {
        println!("{}", prompt);
        let input = read_raw().trim().to_string();
        if !input.is_empty() {
            return input;
        }
        println!("Input cannot be empty. Please try again.");
    }
}

// Reads a number in min..=max, re-asking until valid.
// Every menu uses this, so a wrong choice never kicks the user out;
// the "exit/back/cancel" entry is simply one of the valid numbers.
fn read_number(prompt: &str, min: usize, max: usize) -> usize {
    loop {
        match read_input(prompt).parse::<usize>() {
            Ok(n) if (min..=max).contains(&n) => return n,
            _ => println!("  Please enter a number between {} and {}.", min, max),
        }
    }
}

// Reads multi-line text until a line containing only END. Blank lines are allowed.
fn read_multiline(prompt: &str) -> String {
    println!("{}", prompt);
    println!("(Type END on a new line by itself to finish):");
    let mut content = String::new();
    loop {
        let line = read_raw();
        if line.trim().eq_ignore_ascii_case("END") {
            return content;
        }
        content.push_str(&line);
        content.push('\n');
    }
}

fn confirm(prompt: &str) -> bool {
    read_input(&format!("{} Type YES to confirm, anything else to cancel:", prompt))
        .eq_ignore_ascii_case("YES")
}

// -------------------------------------------------------
// FILE HELPERS
// -------------------------------------------------------
fn note_path(name: &str) -> String {
    format!("{}/{}.txt", NOTES_DIR, name)
}

fn get_all_notes() -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(NOTES_DIR)
        .into_iter()
        .flatten() // Result<ReadDir> -> entries (empty if the folder can't be read)
        .flatten() // drop entries that failed to read
        .filter_map(|e| {
            let file = e.file_name().to_string_lossy().into_owned();
            file.strip_suffix(".txt").map(String::from)
        })
        .collect();
    names.sort();
    names
}

fn print_numbered(items: &[String]) {
    for (i, item) in items.iter().enumerate() {
        println!("  {}. {}", i + 1, item);
    }
}

fn write_file(path: &str, content: &str) -> bool {
    match fs::write(path, content) {
        Ok(_) => true,
        Err(e) => {
            println!("  Failed to save: {}", e);
            false
        }
    }
}

// Lines -> file text. An empty list gives an empty file (not a lone "\n").
fn join_lines(lines: &[String]) -> String {
    if lines.is_empty() { String::new() } else { lines.join("\n") + "\n" }
}

// Asks for a note name until it is valid. Returns None if the user types 0.
// (So "0" cannot be used as a note name.)
fn ask_name(prompt: &str, must_be_new: bool) -> Option<String> {
    loop {
        let raw = read_input(&format!("{} (0 to cancel):", prompt));
        if raw == "0" {
            return None;
        }
        if raw.contains('/') || raw.contains('\\') || raw.contains("..") {
            println!("  Invalid name: no slashes or '..' allowed.");
            continue;
        }
        let name = raw.replace(' ', "_");
        if must_be_new && Path::new(&note_path(&name)).exists() {
            println!("  A note named '{}' already exists. Choose a different name.", name);
            continue;
        }
        return Some(name);
    }
}

// Asks for a name, confirms before overwriting, saves.
fn save_text(label: &str, content: &str) {
    let Some(name) = ask_name(&format!("Enter file name for {}", label), false) else {
        println!("  Not saved.");
        return;
    };
    if Path::new(&note_path(&name)).exists() && !confirm(&format!("'{}' already exists. Overwrite?", name)) {
        println!("  Not saved.");
        return;
    }
    if write_file(&note_path(&name), content) {
        println!("  Saved to '{}.txt'!", name);
    }
}

// Lets the user pick a note by number. Returns None if there are none or 0 is chosen.
fn pick_note(action: &str) -> Option<String> {
    let notes = get_all_notes();
    if notes.is_empty() {
        println!("  No notes exist yet.");
        return None;
    }
    println!("\n  Choose a note to {}:", action);
    print_numbered(&notes);
    match read_number("  Enter number (or 0 to cancel):", 0, notes.len()) {
        0 => None,
        n => notes.into_iter().nth(n - 1),
    }
}

// -------------------------------------------------------
// FEATURES 1-7 — note management
// -------------------------------------------------------
fn create_note() {
    println!("\n--- Create New Note ---");
    let Some(name) = ask_name("Enter note name (e.g. shopping, diary, ideas)", true) else { return };
    let content = read_multiline("Enter the content:");
    if write_file(&note_path(&name), &content) {
        println!("\n  Note '{}' saved successfully!", name);
    }
}

fn list_notes() {
    println!("\n--- All Notes ---");
    let notes = get_all_notes();
    if notes.is_empty() {
        println!("  No notes found. Create one first!");
        return;
    }
    print_numbered(&notes);
    println!("\n  Total: {} note(s)", notes.len());
}

fn read_note() {
    println!("\n--- Read Note ---");
    let Some(name) = pick_note("read") else { return };
    match fs::read_to_string(note_path(&name)) {
        Ok(content) => {
            let bar = "=".repeat(30);
            println!("\n{}\n  NOTE: {}\n{}", bar, name.replace('_', " "), bar);
            println!("{}\n{}\n", content, bar);
        }
        Err(e) => println!("  Failed to read note: {}", e),
    }
}

fn edit_note() {
    println!("\n--- Edit Note ---");
    let Some(name) = pick_note("edit") else { return };
    let path = note_path(&name);
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            println!("  Failed to read note: {}", e);
            return;
        }
    };
    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    println!("Current content:\n---\n{}\n---", content.trim());
    println!("\n  Choose edit mode:");
    println!("  1. Overwrite completely");
    println!("  2. Insert after a specific line (0 = top, last line = append)");
    println!("  3. Delete lines");
    println!("  4. Back to main menu");

    match read_number("  Enter choice:", 1, 4) {
        1 => {
            let new_content = read_multiline("Enter the new content:");
            if write_file(&path, &new_content) {
                println!("\n  Note '{}' overwritten successfully!", name);
            }
        }
        2 => {
            if !lines.is_empty() {
                println!("\n  Lines in this note:");
                print_numbered(&lines);
            }
            let at = read_number(
                &format!("\n  Insert after which line? (0 for top, {} for end):", lines.len()),
                0,
                lines.len(),
            );
            let new_text = read_multiline("  Enter new content to insert.");
            let tail = lines.split_off(at);
            lines.extend(new_text.lines().map(String::from));
            lines.extend(tail);
            if write_file(&path, &join_lines(&lines)) {
                println!("\n  Note '{}' updated successfully!", name);
            }
        }
        3 => loop {
            if lines.is_empty() {
                println!("  No lines left to delete.");
                break;
            }
            println!("\n  Lines in this note:");
            print_numbered(&lines);
            let n = read_number("Enter line number to delete (0 to finish):", 0, lines.len());
            if n == 0 {
                break;
            }
            lines.remove(n - 1);
            if write_file(&path, &join_lines(&lines)) {
                println!("  Line {} removed.", n);
            }
        },
        _ => println!("  Cancelled."),
    }
}

fn rename_note() {
    println!("\n--- Rename Note ---");
    let Some(old_name) = pick_note("rename") else { return };
    let Some(new_name) = ask_name("Enter new name", true) else { return };
    match fs::rename(note_path(&old_name), note_path(&new_name)) {
        Ok(_) => println!("\n  Renamed '{}' to '{}' successfully!", old_name, new_name),
        Err(e) => println!("\n  Failed to rename note: {}", e),
    }
}

fn delete_note() {
    println!("\n--- Delete Note ---");
    let Some(name) = pick_note("delete") else { return };
    if !confirm(&format!("\n  Delete '{}'? This cannot be undone.", name)) {
        println!("  Deletion cancelled.");
        return;
    }
    match fs::remove_file(note_path(&name)) {
        Ok(_) => println!("\n  Note '{}' deleted.", name),
        Err(e) => println!("\n  Failed to delete note: {}", e),
    }
}

fn search_notes() {
    println!("\n--- Search Notes ---");
    let keyword = read_input("Enter keyword to search for:");
    let key = keyword.to_lowercase();
    let notes = get_all_notes();
    if notes.is_empty() {
        println!("  No notes to search.");
        return;
    }
    let bar = "=".repeat(40);
    let mut found_any = false;
    for name in &notes {
        let Ok(content) = fs::read_to_string(note_path(name)) else { continue };
        let hits: Vec<(usize, &str)> = content
            .lines()
            .enumerate()
            .filter(|(_, l)| l.to_lowercase().contains(&key))
            .collect();
        if hits.is_empty() {
            continue;
        }
        if !found_any {
            println!("\n  Results for '{}':\n  {}", keyword, bar);
            found_any = true;
        }
        println!("\n  Found in note: {}", name.replace('_', " "));
        for (i, line) in hits {
            println!("    Line {}: {}", i + 1, line.trim());
        }
    }
    if found_any {
        println!("\n  {}", bar);
    } else {
        println!("\n  No notes contain '{}'.", keyword);
    }
}

// -------------------------------------------------------
// FEATURE 8 — Filter text into categories
//
//   0 Number        digits only                   42
//   1 IP Address    digits + dots only            192.168.0.1  (decimals land here too)
//   2 Web Address   letters AND a dot             www.example.com
//   3 Alphanumeric  letters AND digits, no dot    MI6, ABC123   ("hello" is NOT included)
//
// Punctuation around a token is trimmed first, so "end." or "(MI6)," are judged
// as "end" and "MI6" — otherwise a sentence-ending "end." would look like a web address.
// After trimming, the first and last characters are letters/digits, which is why
// a token like "..." can never reach the digit/dot test.
// -------------------------------------------------------
fn classify(token: &str) -> Option<(usize, &str)> {
    let t = token.trim_matches(|c: char| !c.is_alphanumeric());
    if t.is_empty() {
        return None;
    }
    let has_dot = t.contains('.');
    let has_digit = t.chars().any(|c| c.is_ascii_digit());
    let has_letter = t.chars().any(|c| c.is_alphabetic());
    // The "exclusive" guard: ONLY digits and dots. Without it, "abc1.2" would
    // look like an IP, because has_dot and has_digit alone don't rule out letters.
    let only_digits_dots = t.chars().all(|c| c.is_ascii_digit() || c == '.');

    let category = if only_digits_dots {
        if has_dot { 1 } else { 0 }
    } else if has_letter && has_dot {
        2
    } else if has_letter && has_digit {
        3
    } else {
        return None; // plain words like "hello", or mixed symbols like "42,000"
    };
    Some((category, t))
}

// Text for one category: items one per line, ending with exactly one "\n".
// (join only puts "\n" BETWEEN items, so the final item needs this closing one.)
fn category_body(items: &[String]) -> String {
    if items.is_empty() { "(none)\n".to_string() } else { items.join("\n") + "\n" }
}

fn choose_categories() -> Option<Vec<usize>> {
    loop {
        println!("\n  1. Numbers   2. IP Addresses   3. Web Addresses   4. Alphanumeric Codes");
        let raw = read_input("  Enter numbers separated by spaces (or 'c' to go back):");
        if raw.eq_ignore_ascii_case("c") {
            return None;
        }
        let mut chosen: Vec<usize> = raw
            .split_whitespace()
            .filter_map(|s| s.parse::<usize>().ok())
            .filter(|n| (1..=4).contains(n))
            .map(|n| n - 1)
            .collect();
        chosen.sort();
        chosen.dedup();
        if chosen.is_empty() {
            println!("  No valid selections. Enter numbers 1-4 or 'c' to go back.");
        } else {
            return Some(chosen);
        }
    }
}

// Returns false if the user backed out, so the caller can show the action menu again.
fn save_groups(indices: &[usize], groups: &[Vec<String>; 4]) -> bool {
    println!("\nSave as:");
    println!("  1. Single file (all selected categories)");
    println!("  2. Separate files (one per category)");
    println!("  3. Back");
    match read_number("Enter choice:", 1, 3) {
        1 => {
            let mut text = String::new();
            for &i in indices {
                // header, items (each ending in \n), then one extra \n = blank separator line
                text.push_str(&format!("--- {} ---\n{}\n", LABELS[i], category_body(&groups[i])));
            }
            save_text("the results", &text);
            true
        }
        2 => {
            for &i in indices {
                save_text(LABELS[i], &category_body(&groups[i]));
            }
            true
        }
        _ => false,
    }
}

fn filter() {
    println!("\n--- Filter Text ---");
    println!("Categorises tokens: Numbers, IP Addresses, Web Addresses, Alphanumeric Codes.");
    println!("(Decimal point numbers are treated as IP addresses.)");

    let content = read_multiline("Enter the text to filter:");

    let mut groups: [Vec<String>; 4] = Default::default();
    for token in content.split_whitespace() {
        if let Some((category, clean)) = classify(token) {
            groups[category].push(clean.to_string());
        }
    }

    println!("\n========== FILTER RESULTS ==========");
    for (label, items) in LABELS.iter().zip(&groups) {
        println!("\n--- {} ---", label);
        if items.is_empty() {
            println!("  (none found)");
        } else {
            for item in items {
                println!("  {}", item);
            }
        }
    }
    println!("\n=====================================");

    loop {
        println!("\nWhat would you like to do?");
        println!("  1. Save all categories");
        println!("  2. Save specific categories");
        println!("  3. Exit (don't save)");

        let indices = match read_number("Enter choice:", 1, 3) {
            1 => vec![0, 1, 2, 3],
            2 => match choose_categories() {
                Some(v) => v,
                None => continue,
            },
            _ => return,
        };
        if save_groups(&indices, &groups) {
            return;
        }
    }
}

// -------------------------------------------------------
// FEATURE 9 — Reverse text
//   1: whole text reversed   "hello world" -> "dlrow olleh"
//   2: each word reversed    "hello world" -> "olleh dlrow"  (line breaks kept)
// -------------------------------------------------------
fn reverse_text() {
    println!("\n--- Reverse Text ---");
    println!("  1. Reverse entire text (all characters)");
    println!("  2. Reverse each word's characters (word order preserved)");
    println!("  3. Cancel");
    let mode = read_number("Enter choice:", 1, 3);
    if mode == 3 {
        return;
    }

    let content = read_multiline("Enter the text to reverse:");
    let text = content.trim();

    let result: String = if mode == 1 {
        text.chars().rev().collect()
    } else {
        text.lines()
            .map(|line| {
                line.split_whitespace()
                    .map(|w| w.chars().rev().collect::<String>())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    println!("\n--- Result ---\n{}\n{}", result, "=".repeat(30));

    println!("\nWhat would you like to do?");
    println!("  1. Save result");
    println!("  2. Exit");
    if read_number("Enter choice:", 1, 2) == 1 {
        save_text("the result", &format!("{}\n", result));
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

    let bar = "=".repeat(45);
    println!("{}\n              NOTEPAD CLI\n  Notes are saved in the 'notes/' folder\n{}\n", bar, bar);

    loop {
        println!("========== MAIN MENU ==========");
        println!("1.  Create new note");
        println!("2.  List all notes");
        println!("3.  Read a note");
        println!("4.  Edit a note");
        println!("5.  Rename a note");
        println!("6.  Delete a note");
        println!("7.  Search notes");
        println!("8.  Filter text");
        println!("9.  Reverse text");
        println!("10. Exit");
        println!("{}", "=".repeat(30));

        match read_number("Choose an option:", 1, 10) {
            1 => create_note(),
            2 => list_notes(),
            3 => read_note(),
            4 => edit_note(),
            5 => rename_note(),
            6 => delete_note(),
            7 => search_notes(),
            8 => filter(),
            9 => reverse_text(),
            _ => {
                println!("\nGoodbye! Your notes are saved in the 'notes/' folder.");
                break;
            }
        }
    }
}
