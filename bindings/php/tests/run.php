<?php
// Run with BDHTML2PDF_BINARY pointing to a built bdhtml2pdf: php tests/run.php

declare(strict_types=1);

spl_autoload_register(function (string $class): void {
    $file = __DIR__ . '/../src/' . substr($class, strlen('Bdhtml2pdf\\')) . '.php';
    if (str_starts_with($class, 'Bdhtml2pdf\\') && is_file($file)) {
        require $file;
    }
});

use Bdhtml2pdf\Bdhtml2pdfException;
use Bdhtml2pdf\Client;
use Bdhtml2pdf\Converter;
use Bdhtml2pdf\Options;

const PAGE = '<!DOCTYPE html><p id="p"></p><script>p.textContent = "Merhaba İğdır"</script>';
$failed = 0;
$passed = 0;

function check(string $name, bool $ok): void
{
    global $failed, $passed;
    if ($ok) {
        $passed++;
    } else {
        $failed++;
        echo "FAIL $name\n";
    }
}

function throws(callable $f, string $class, ?int $code = null, ?string $pattern = null): bool
{
    try {
        $f();
    } catch (\Throwable $e) {
        return $e instanceof $class
            && ($code === null || $e->getCode() === $code)
            && ($pattern === null || preg_match($pattern, $e->getMessage()) === 1);
    }
    return false;
}

$c = new Converter();
$pdf = $c->convert(PAGE, ['page_size' => 'A5', 'title' => 'Fatura']);
check('pdf', str_starts_with($pdf, '%PDF-') && str_contains($pdf, '419.52'));
check('html', str_contains($c->convert(PAGE, [], 'html'), '<p id="p">Merhaba İğdır'));
check('png', str_starts_with($c->convert(PAGE, ['width' => 300, 'height' => 200], 'png'), "\x89PNG"));
check('pdf-a', str_contains($c->convert('<title>T</title><p>x</p>', ['pdfA' => '2b']), '<pdfaid:part>2</pdfaid:part>'));
check('flags', Options::flags(['disable_javascript' => true, 'quiet' => false, 'title' => null, 'zoom' => 2, 'pageSize' => 'A5'])
    === ['--disable-javascript', '--zoom=2', '--page-size=A5']);
$big = str_repeat('<p>Satır ğüşiöç</p>', 50000);
check('large page through the pipes', str_starts_with($c->convert($big, ['disable_javascript' => true]), '%PDF-'));

$dir = sys_get_temp_dir() . '/bdhtml2pdf-php-' . getmypid();
@mkdir($dir);
file_put_contents("$dir/s.js", 'document.body.textContent = "from s.js"');
check('base dir', str_contains((new Converter(null, $dir))->convert('<body><script src="s.js"></script>', [], 'html'), 'from s.js'));
unlink("$dir/s.js");
rmdir($dir);

check('error message', throws(fn () => $c->convert('<p>x</p>', ['page_size' => 'Z9']), Bdhtml2pdfException::class, null, '/unknown page size/'));
check('bad format', throws(fn () => $c->convert('<p>x</p>', [], 'docx'), \InvalidArgumentException::class));

// Server
$bin = getenv('BDHTML2PDF_BINARY') ?: 'bdhtml2pdf';
$pipes = [];
$proc = proc_open([$bin, 'serve', '--listen', '127.0.0.1:0', '--jobs', '2', '--max-memory', '200'], [2 => ['pipe', 'w']], $pipes);
preg_match('{(http://\S+)}', (string) fgets($pipes[2]), $m);
$client = new Client($m[1]);
check('health', str_starts_with($client->health(), 'ok bdhtml2pdf'));
check('server pdf', str_contains($client->convert(PAGE, ['page_size' => 'A5']), '419.52'));
check('server bool', str_contains($client->convert(PAGE, ['disableJavascript' => true], 'html'), '<p id="p"></p>'));
check('server 400', throws(fn () => $client->convert('<p>x</p>', ['max_memory' => 0]), Bdhtml2pdfException::class, 400));
check('server 507', throws(fn () => $client->convert('<script>let s = "x"; for (;;) s = s + s;</script>'), Bdhtml2pdfException::class, 507));
proc_terminate($proc, 9);
proc_close($proc);

echo "$passed passed, $failed failed\n";
exit($failed === 0 ? 0 : 1);
