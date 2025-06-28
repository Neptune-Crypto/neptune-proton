#!/usr/bin/env php
<?php

// Acknowledge user preference for function naming style for future use if needed.
function process_rust_trait($file_path) {
    if (!file_exists($file_path) || !is_readable($file_path)) {
        echo "Error: File not found or is not readable: " . $file_path . "\n";
        exit(1);
    }

    $lines = file($file_path);
    if ($lines === false) {
        echo "Error: Could not read file: " . $file_path . "\n";
        exit(1);
    }

    $trait_code = '';
    $is_capturing = false;
    $brace_level = 0;
    $found_tarpc_attribute = false;
    $initial_trait_indentation = '';

    foreach ($lines as $line) {
        if (strpos($line, '#[tarpc::service]') !== false) {
            $found_tarpc_attribute = true;
        }

        if ($found_tarpc_attribute && !$is_capturing && strpos($line, 'trait') !== false) {
            $is_capturing = true;
            // Capture the indentation of the trait keyword itself
            preg_match('/^(\s*)/', $line, $indent_match);
            $initial_trait_indentation = $indent_match[1] ?? '';
        }

        if ($is_capturing) {
            $trait_code .= $line;
            $brace_level += substr_count($line, '{');
            $brace_level -= substr_count($line, '}');

            if ($brace_level == 0 && strpos($trait_code, '{') !== false) {
                break;
            }
        }
    }

    if (empty($trait_code)) {
        echo "Error: No trait with #[tarpc::service] found in " . $file_path . "\n";
        exit(1);
    }

    // --- Start Processing the Captured Trait ---

    // 1. Strip out all block comments (/* ... */)
    $processed_code = preg_replace('/\/\*.*?\*\//s', '', $trait_code);

    // 2. Strip out all single-line non-doc comments (// ...)
    $processed_code = preg_replace('/^\s*\/\/(?!\/).*?$/m', '', $processed_code);

    // 3. Collapse multi-line doc-comments (///) to a single line
    $processed_code = preg_replace_callback(
        '/(?m)(?:^\s*\/\/\/[^\n]*\n)+/',
        function ($matches) {
            $block = $matches[0];
            if (preg_match('/^\s*\/\/\/(.*)/', $block, $first_line_matches)) {
                return '    ///' . $first_line_matches[1] . "\n";
            }
            return $block;
        },
        $processed_code
    );

    // 4. Ensure a single blank line after each method (ending with ';')
    $processed_code = preg_replace('/;\s*$/m', ";\n", $processed_code);

    // 5. Clean up excessive blank lines, ensuring just one between methods
    $processed_code = preg_replace('/\n\s*\n/', "\n\n", $processed_code);

    // Reconstruct the final output
    $final_output = "#[tarpc::service]\n" . ltrim($processed_code);

    echo "// This file is auto-generated from " . basename($file_path) . ". Do not edit directly.\n\n";
    echo $final_output;
}

if ($argc < 2) {
    echo "Usage: php " . $argv[0] . " <path_to_rust_file>\n";
    exit(1);
}

$rust_file_path = $argv[1];
process_rust_trait($rust_file_path);

?>