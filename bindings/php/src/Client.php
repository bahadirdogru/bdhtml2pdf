<?php

declare(strict_types=1);

namespace Bdhtml2pdf;

/**
 * A client for `bdhtml2pdf serve`. Needs allow_url_fopen; no extensions.
 *
 * The server loads nothing from its disk, so a page carries its images and
 * styles inline (data: URLs, <style>).
 */
final class Client
{
    private string $url;

    public function __construct(string $url = 'http://127.0.0.1:7368', private float $timeout = 60)
    {
        $this->url = rtrim($url, '/');
    }

    public function health(): string
    {
        return trim($this->request('GET', '/health', null));
    }

    /** @param array<string, scalar|null> $options flags */
    public function convert(string $html, array $options = [], string $format = 'pdf'): string
    {
        $query = [];
        foreach ($options as $key => $value) {
            if ($value === null || $value === false) {
                continue;
            }
            $query[Options::name($key)] = $value === true ? 'true' : (string) $value;
        }
        $path = '/' . Options::format($format) . ($query ? '?' . http_build_query($query) : '');
        return $this->request('POST', $path, $html);
    }

    private function request(string $method, string $path, ?string $body): string
    {
        $http = ['method' => $method, 'timeout' => $this->timeout, 'ignore_errors' => true];
        if ($body !== null) {
            $http['header'] = "Content-Type: text/html; charset=utf-8\r\n";
            $http['content'] = $body;
        }
        $result = @file_get_contents($this->url . $path, false, stream_context_create(['http' => $http]));
        $headers = function_exists('http_get_last_response_headers')
            ? (http_get_last_response_headers() ?? [])
            : ($http_response_header ?? []);
        if ($result === false || $headers === []) {
            throw new Bdhtml2pdfException("cannot reach {$this->url}");
        }
        preg_match('{^HTTP/\S+ (\d+)}', $headers[0], $m);
        $code = (int) ($m[1] ?? 0);
        if ($code !== 200) {
            throw new Bdhtml2pdfException(Options::message($result, "HTTP $code"), $code);
        }
        return $result;
    }
}
