<?php
/**
 * OxideAdmin Universal Process Supervisor & Streaming Reverse Proxy Bridge
 *
 * Enables running standalone Rust binaries on 100% of standard cPanel & Apache
 * shared hosting accounts without root access, custom Daemons, or systemd.
 *
 * @version 1.0.0
 * @author OxideAdmin Open Source
 */

declare(strict_types=1);

// Configuration
$appPort    = 3000;
$appHost    = '127.0.0.1';
$binaryName = 'app';

// Paths relative to public_html/
$appDir     = dirname(__DIR__) . '/app';
$binaryPath = $appDir . '/' . $binaryName;
$logPath    = $appDir . '/oxide.log';
$pidPath    = $appDir . '/oxide.pid';

// -----------------------------------------------------------------------------
// 1. Process Supervisor: Check if Rust binary is listening; if not, boot it
// -----------------------------------------------------------------------------
function isPortOpen(string $host, int $port, float $timeout = 0.15): bool {
    $fp = @fsockopen($host, $port, $errno, $errstr, $timeout);
    if ($fp) {
        fclose($fp);
        return true;
    }
    return false;
}

if (!isPortOpen($appHost, $appPort)) {
    if (file_exists($binaryPath)) {
        if (!is_executable($binaryPath)) {
            @chmod($binaryPath, 0755);
        }

        // Launch binary in background with unbuffered output
        $cmd = sprintf(
            'PORT=%d %s >> %s 2>&1 & echo $!',
            $appPort,
            escapeshellarg($binaryPath),
            escapeshellarg($logPath)
        );
        $pid = trim((string) @shell_exec($cmd));
        if ($pid !== '') {
            @file_put_contents($pidPath, $pid);
        }

        // Wait up to 500ms for Rust to bind socket (typically starts in <5ms)
        $booted = false;
        for ($i = 0; $i < 10; $i++) {
            usleep(50000); // 50ms
            if (isPortOpen($appHost, $appPort)) {
                $booted = true;
                break;
            }
        }

        if (!$booted && !isPortOpen($appHost, $appPort)) {
            http_response_code(502);
            header('Content-Type: text/html; charset=utf-8');
            $logContent = file_exists($logPath) ? htmlspecialchars((string) file_get_contents($logPath)) : 'No log output generated.';
            echo '<!DOCTYPE html><html><head><title>OxideAdmin Boot Error</title>';
            echo '<meta name="viewport" content="width=device-width, initial-scale=1">';
            echo '<style>body{font-family:ui-sans-serif,system-ui,sans-serif;background:#0f172a;color:#f8fafc;padding:2rem;}';
            echo '.card{max-width:680px;margin:2rem auto;background:#1e293b;border:1px solid #334155;border-radius:12px;padding:2rem;}';
            echo 'h1{color:#38bdf8;font-size:1.5rem;margin-top:0;}pre{background:#090d16;padding:1rem;border-radius:8px;overflow:auto;color:#cbd5e1;font-size:0.875rem;}';
            echo '.tip{color:#94a3b8;font-size:0.875rem;margin-top:1rem;border-left:3px solid #38bdf8;padding-left:0.75rem;}';
            echo '</style></head><body><div class="card">';
            echo '<h1>⚡ OxideAdmin Process Supervisor</h1>';
            echo '<p>Unable to connect to the OxideAdmin binary on <code>127.0.0.1:' . $appPort . '</code>.</p>';
            echo '<div class="tip">Make sure <code>app/' . htmlspecialchars($binaryName) . '</code> has execute permissions (<code>chmod +x</code> or <code>755</code>).</div>';
            echo '<h3>Recent Application Logs:</h3><pre>' . $logContent . '</pre>';
            echo '</div></body></html>';
            exit;
        }
    }
}

// -----------------------------------------------------------------------------
// 2. High-Performance Streaming HTTP Reverse Proxy
// -----------------------------------------------------------------------------
$requestUri = $_SERVER['REQUEST_URI'] ?? '/';
$targetUrl  = sprintf('http://%s:%d%s', $appHost, $appPort, $requestUri);

$method     = $_SERVER['REQUEST_METHOD'] ?? 'GET';
$requestBody = file_get_contents('php://input');

// Forward incoming HTTP headers
$forwardHeaders = [];
foreach ($_SERVER as $key => $value) {
    if (strpos($key, 'HTTP_') === 0) {
        $headerName = str_replace(' ', '-', ucwords(strtolower(str_replace('_', ' ', substr($key, 5)))));
        if (!in_array(strtolower($headerName), ['host', 'connection', 'content-length'], true)) {
            $forwardHeaders[] = "$headerName: $value";
        }
    }
}

// Ensure Host & Forward headers match original client request
$forwardHeaders[] = 'Host: ' . ($_SERVER['HTTP_HOST'] ?? 'localhost');
$forwardHeaders[] = 'X-Forwarded-For: ' . ($_SERVER['REMOTE_ADDR'] ?? '127.0.0.1');
$forwardHeaders[] = 'X-Forwarded-Proto: ' . ((!empty($_SERVER['HTTPS']) && $_SERVER['HTTPS'] !== 'off') ? 'https' : 'http');

if (isset($_SERVER['CONTENT_TYPE'])) {
    $forwardHeaders[] = 'Content-Type: ' . $_SERVER['CONTENT_TYPE'];
}

$ch = curl_init($targetUrl);
curl_setopt($ch, CURLOPT_CUSTOMREQUEST, $method);
curl_setopt($ch, CURLOPT_HTTPHEADER, $forwardHeaders);
curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
curl_setopt($ch, CURLOPT_HEADER, true);
curl_setopt($ch, CURLOPT_FOLLOWLOCATION, false);
curl_setopt($ch, CURLOPT_TIMEOUT, 60);

if (in_array($method, ['POST', 'PUT', 'PATCH', 'DELETE'], true) && !empty($requestBody)) {
    curl_setopt($ch, CURLOPT_POSTFIELDS, $requestBody);
}

$response = curl_exec($ch);

if ($response === false) {
    http_response_code(502);
    header('Content-Type: text/plain');
    echo 'Bad Gateway: OxideAdmin service unreachable (' . curl_error($ch) . ')';
    curl_close($ch);
    exit;
}

$headerSize = curl_getinfo($ch, CURLINFO_HEADER_SIZE);
$httpCode   = curl_getinfo($ch, CURLINFO_HTTP_CODE);
curl_close($ch);

$rawHeaders = substr($response, 0, $headerSize);
$body       = substr($response, $headerSize);

// Send status code
http_response_code($httpCode);

// Forward response headers (handling multiple Set-Cookie headers properly)
$headers = explode("\r\n", $rawHeaders);
foreach ($headers as $header) {
    if (trim($header) === '' || stripos($header, 'HTTP/') === 0) {
        continue;
    }
    if (stripos($header, 'Transfer-Encoding:') === 0) {
        continue;
    }
    header($header, false);
}

echo $body;
