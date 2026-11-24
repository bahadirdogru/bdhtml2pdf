<?php

declare(strict_types=1);

namespace Bdhtml2pdf;

/** @internal Option names and formats shared by Converter and Client. */
final class Options
{
    public const FORMATS = ['pdf', 'png', 'jpg', 'html'];

    /** page_size, pageSize → page-size */
    public static function name(string $key): string
    {
        return strtolower((string) preg_replace('/(?<!^)[A-Z]/', '-$0', str_replace('_', '-', $key)));
    }

    /**
     * Options as command-line flags. true is a bare flag; false and null leave
     * the flag out. Values use --flag=value, so a value that starts with a dash
     * stays a value.
     *
     * @param array<string, scalar|null> $options
     * @return list<string>
     */
    public static function flags(array $options): array
    {
        $out = [];
        foreach ($options as $key => $value) {
            if ($value === null || $value === false) {
                continue;
            }
            $out[] = $value === true ? '--' . self::name($key) : '--' . self::name($key) . '=' . $value;
        }
        return $out;
    }

    public static function format(string $format): string
    {
        $f = strtolower($format);
        if ($f === 'jpeg') {
            $f = 'jpg';
        }
        if (!in_array($f, self::FORMATS, true)) {
            throw new \InvalidArgumentException("unknown format \"$format\"; expected one of " . implode(', ', self::FORMATS));
        }
        return $f;
    }

    public static function message(string $text, string $fallback): string
    {
        $text = trim($text);
        return (string) preg_replace('/^error: /', '', $text === '' ? $fallback : $text);
    }
}
