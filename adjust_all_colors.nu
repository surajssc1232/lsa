#!/usr/bin/env nu

# Read the themes.rs file
let content = open src/themes.rs

# Function to adjust color
def adjust_color [r: int, g: int, b: int] {
    let avg = ($r + $g + $b) / 3.0
    if $avg >= 150 {
        [$r, $g, $b]
    } else {
        if $avg == 0 {
            [150, 150, 150]
        } else {
            let factor = 150.0 / $avg
            let nr = (($r * $factor) | into int | if ($in > 255) { 255 } else { $in })
            let ng = (($g * $factor) | into int | if ($in > 255) { 255 } else { $in })
            let nb = (($b * $factor) | into int | if ($in > 255) { 255 } else { $in })
            [$nr, $ng, $nb]
        }
    }
}

# Replace all color lines
$content | lines | each { |line|
    if ($line | str contains ": (") {
        let captures = ($line | parse -r ': \((\d+), (\d+), (\d+)\)')
        if ($captures | length) > 0 {
            let r = ($captures | get 0 | get capture0 | into int)
            let g = ($captures | get 0 | get capture1 | into int)
            let b = ($captures | get 0 | get capture2 | into int)
            let new_rgb = (adjust_color $r $g $b)
            let nr = ($new_rgb | get 0)
            let ng = ($new_rgb | get 1)
            let nb = ($new_rgb | get 2)
            let replacement = (": (" + ($nr | into string) + ", " + ($ng | into string) + ", " + ($nb | into string) + ")")
            $line | str replace -r ': \([0-9]+, [0-9]+, [0-9]+\)' $replacement
        } else {
            $line
        }
    } else {
        $line
    }
} | str join "\n" | save -f src/themes.rs
