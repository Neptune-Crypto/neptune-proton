#!/usr/bin/env php
<?php

// Adhering to the requested lower_case_style for all functions and variables.

/**
 * Normalizes a Rust method signature for consistent comparison.
 * - Removes leading/trailing whitespace.
 * - Removes 'async' and 'pub' keywords.
 * - Collapses multiple whitespace characters into a single space.
 *
 * @param string $signature The raw signature string.
 * @return string The normalized signature.
 */
function normalize_signature($signature) {
    // Remove keywords that don't affect the core signature
    $signature = str_replace(['async ', 'pub '], '', $signature);
    // Collapse all contiguous whitespace into a single space
    $signature = preg_replace('/\s+/', ' ', $signature);
    return trim($signature);
}

/**
 * Extracts all method signatures from a #[tarpc::service] trait in a given Rust file.
 *
 * @param string $file_path The path to the Rust file.
 * @return array An associative array mapping method names to their normalized signatures.
 */
function get_trait_methods($file_path) {
    if (!file_exists($file_path) || !is_readable($file_path)) {
        echo "Error: File not found or is not readable: " . $file_path . "\n";
        exit(1);
    }

    $raw_lines = file($file_path, FILE_IGNORE_NEW_LINES);
    if ($raw_lines === false) {
        echo "Error: Could not read file: " . $file_path . "\n";
        exit(1);
    }

    // --- 1. Pre-processing stage: Remove all slash-style comments ---
    $lines = [];
    foreach ($raw_lines as $line) {
        $comment_position = strpos($line, '//');
        if ($comment_position !== false) {
            $lines[] = substr($line, 0, $comment_position);
        } else {
            $lines[] = $line;
        }
    }

    // --- 2. Extract the full trait block using brace counting on cleaned lines ---
    $trait_code = '';
    $is_capturing = false;
    $brace_level = 0;
    $found_tarpc_attribute = false;

    foreach ($lines as $line) {
        if (strpos($line, '#[tarpc::service]') !== false) {
            $found_tarpc_attribute = true;
        }

        if ($found_tarpc_attribute && !$is_capturing && strpos($line, 'trait') !== false) {
            $is_capturing = true;
        }

        if ($is_capturing) {
            $trait_code .= $line . "\n";
            $brace_level += substr_count($line, '{');
            $brace_level -= substr_count($line, '}');

            if ($brace_level == 0 && strpos($trait_code, '{') !== false) {
                break;
            }
        }
    }

    if (empty($trait_code)) {
        echo "Error: No trait with #[tarpc::service] found in " . $file_path . "\n";
        return [];
    }

    // --- 3. Isolate just the content between the braces ---
    $body_start = strpos($trait_code, '{');
    $body_end = strrpos($trait_code, '}');
    if ($body_start === false || $body_end === false) {
        return [];
    }
    $trait_body = substr($trait_code, $body_start + 1, $body_end - $body_start - 1);

    // --- 4. Find all function signatures from the comment-free body ---
    $methods = [];
    // This regex captures the function name and its full signature line.
    // THE FIX: Added 's' flag to make '.' match newlines for multi-line signatures.
    preg_match_all('/^\s*(?:async\s+)?(?:pub\s+)?fn\s+([a-zA-Z0-9_]+)\s*\(.*?;/ms', $trait_body, $matches, PREG_SET_ORDER);

    foreach ($matches as $match) {
        $method_name = $match[1];
        $full_signature = $match[0];
        $methods[$method_name] = normalize_signature($full_signature);
    }

    return $methods;
}

// --- Main Script Execution ---

if ($argc !== 3) {
    echo "Usage: php " . $argv[0] . " <file1.rs> <file2.rs>\n";
    exit(1);
}

$file1_path = $argv[1];
$file2_path = $argv[2];

$methods1 = get_trait_methods($file1_path);
$methods2 = get_trait_methods($file2_path);

$new_or_missing = [];
$modified = [];
$removed = [];

// --- Compare the method sets ---

// Find new/missing and modified methods
foreach ($methods1 as $name => $signature) {
    if (!array_key_exists($name, $methods2)) {
        $new_or_missing[] = $signature;
    } elseif ($signature !== $methods2[$name]) {
        $modified[$name] = [
            'file1' => $signature,
            'file2' => $methods2[$name]
        ];
    }
}

// Find removed methods
foreach ($methods2 as $name => $signature) {
    if (!array_key_exists($name, $methods1)) {
        $removed[] = $signature;
    }
}

// --- Generate the Report ---

echo "RPC Trait Difference Report\n";
echo "=============================\n";
echo "File 1: " . basename($file1_path) . "\n";
echo "File 2: " . basename($file2_path) . "\n";
echo "=============================\n\n";

// Section 1: NEW / MISSING METHODS
echo "1. NEW / MISSING METHODS (in " . basename($file1_path) . ", not " . basename($file2_path) . ")\n";
echo "--------------------------------------------------\n";
if (empty($new_or_missing)) {
    echo "   None\n";
} else {
    foreach ($new_or_missing as $signature) {
        echo "   + " . $signature . "\n";
    }
}
echo "\n";

// Section 2: MODIFIED METHODS
echo "2. MODIFIED METHODS\n";
echo "--------------------------------------------------\n";
if (empty($modified)) {
    echo "   None\n";
} else {
    foreach ($modified as $name => $signatures) {
        echo "   ~ Method '" . $name . "' was modified:\n";
        echo "     - " . $signatures['file1'] . " (from " . basename($file1_path) . ")\n";
        echo "     + " . $signatures['file2'] . " (from " . basename($file2_path) . ")\n\n";
    }
}
echo "\n";

// Section 3: REMOVED METHODS
echo "3. REMOVED METHODS (in " . basename($file2_path) . ", not " . basename($file1_path) . ")\n";
echo "--------------------------------------------------\n";
if (empty($removed)) {
    echo "   None\n";
} else {
    foreach ($removed as $signature) {
        echo "   - " . $signature . "\n";
    }
}
echo "\n";
?>
