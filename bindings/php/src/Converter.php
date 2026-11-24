<?php

declare(strict_types=1);

namespace Bdhtml2pdf;

/**
 * Runs the bdhtml2pdf program, one process per page.
 *
 *     $pdf = (new Converter())->convert($html, ['page_size' => 'A5', 'pdf_a' => '2b']);
 *
 * Options are the command-line flags in snake_case or camelCase.
 */
final class Converter
{
    private string $binary;

    /**
     * @param string|null $binary path to bdhtml2pdf; default $BDHTML2PDF_BINARY, then PATH
     * @param string|null $baseDir where relative images, styles and scripts load from
     */
    public function __construct(?string $binary = null, private ?string $baseDir = null)
    {
        $this->binary = $binary ?? self::find();
    }

    private static function find(): string
    {
        $env = getenv('BDHTML2PDF_BINARY');
        if (is_string($env) && $env !== '') {
            return $env;
        }
        $exe = PHP_OS_FAMILY === 'Windows' ? 'bdhtml2pdf.exe' : 'bdhtml2pdf';
        foreach (explode(PATH_SEPARATOR, (string) getenv('PATH')) as $dir) {
            if ($dir !== '' && is_file($dir . DIRECTORY_SEPARATOR . $exe)) {
                return $dir . DIRECTORY_SEPARATOR . $exe;
            }
        }
        throw new Bdhtml2pdfException('bdhtml2pdf not found; install it, put it on PATH or set BDHTML2PDF_BINARY');
    }

    /**
     * Convert $html and return the PDF (or PNG, JPEG, HTML) bytes.
     *
     * @param array<string, scalar|null> $options flags
     */
    public function convert(string $html, array $options = [], string $format = 'pdf'): string
    {
        $cmd = [$this->binary, '-q', '--format=' . Options::format($format), ...Options::flags($options), '-', '-'];
        $pipes = [];
        $proc = proc_open($cmd, [0 => ['pipe', 'r'], 1 => ['pipe', 'w'], 2 => ['pipe', 'w']], $pipes, $this->baseDir);
        if (!is_resource($proc)) {
            throw new Bdhtml2pdfException("cannot run {$this->binary}");
        }
        // Read and write together, so a large page or PDF cannot fill a pipe and stall.
        stream_set_blocking($pipes[0], false);
        stream_set_blocking($pipes[1], false);
        stream_set_blocking($pipes[2], false);
        $out = '';
        $err = '';
        $written = 0;
        $len = strlen($html);
        if ($len === 0) {
            fclose($pipes[0]);
        }
        while (true) {
            $read = [];
            foreach ([1, 2] as $i) {
                if (!feof($pipes[$i])) {
                    $read[] = $pipes[$i];
                }
            }
            $write = $written < $len ? [$pipes[0]] : [];
            if ($read === [] && $write === []) {
                break;
            }
            $except = null;
            if (stream_select($read, $write, $except, 1) === false) {
                break;
            }
            foreach ($write as $w) {
                $n = fwrite($w, substr($html, $written, 65536));
                if ($n === false || $n === 0) {
                    $written = $len; // the child stopped reading
                } else {
                    $written += $n;
                }
                if ($written >= $len) {
                    fclose($pipes[0]);
                }
            }
            foreach ($read as $r) {
                $chunk = (string) fread($r, 65536);
                if ($r === $pipes[1]) {
                    $out .= $chunk;
                } else {
                    $err .= $chunk;
                }
            }
        }
        fclose($pipes[1]);
        fclose($pipes[2]);
        $status = proc_close($proc);
        if ($status !== 0) {
            throw new Bdhtml2pdfException(Options::message($err, "exit status $status"), $status);
        }
        return $out;
    }
}
