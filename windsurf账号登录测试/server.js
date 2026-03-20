// ============================================================
// Node.js Firebase 代理服务器
// 部署在国外服务器上，国内 Python 脚本通过此代理访问 Firebase
// ============================================================

const http = require('http');
const https = require('https');
const url = require('url');

// ============================================================
// 配置
// ============================================================
const PORT = process.env.PORT || 3000;
const FIREBASE_HOST = 'identitytoolkit.googleapis.com';

// ============================================================
// 创建 HTTP 服务器
// ============================================================
const server = http.createServer((req, res) => {
    // 设置 CORS 头
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type, X-Client-Version');
    
    // 处理 OPTIONS 预检请求
    if (req.method === 'OPTIONS') {
        res.writeHead(204);
        res.end();
        return;
    }
    
    // 解析请求 URL
    const parsedUrl = url.parse(req.url, true);
    const pathname = parsedUrl.pathname;
    const search = parsedUrl.search || '';
    
    // 根路径返回状态信息
    if (pathname === '/' || pathname === '') {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({
            status: 'ok',
            message: 'Firebase Proxy is running (Node.js)',
            usage: '访问 /v1/accounts:signInWithPassword?key=YOUR_API_KEY',
            server: 'Node.js ' + process.version
        }));
        return;
    }
    
    // 构建目标 URL
    const targetPath = pathname + search;
    
    console.log(`[${new Date().toISOString()}] ${req.method} ${targetPath}`);
    
    // 收集请求体
    let body = '';
    req.on('data', chunk => {
        body += chunk.toString();
    });
    
    req.on('end', () => {
        // 构建 HTTPS 请求选项
        const options = {
            hostname: FIREBASE_HOST,
            port: 443,
            path: targetPath,
            method: req.method,
            headers: {
                'Content-Type': 'application/json',
                'Content-Length': Buffer.byteLength(body)
            }
        };
        
        // 如果有 X-Client-Version 头，保留它
        if (req.headers['x-client-version']) {
            options.headers['X-Client-Version'] = req.headers['x-client-version'];
        }
        
        // 发送请求到 Firebase
        const proxyReq = https.request(options, (proxyRes) => {
            let responseBody = '';
            
            proxyRes.on('data', chunk => {
                responseBody += chunk.toString();
            });
            
            proxyRes.on('end', () => {
                // 返回响应
                res.writeHead(proxyRes.statusCode, {
                    'Content-Type': 'application/json',
                    'Access-Control-Allow-Origin': '*'
                });
                res.end(responseBody);
                
                // 日志
                const logStatus = proxyRes.statusCode === 200 ? '✅' : '❌';
                console.log(`    ${logStatus} ${proxyRes.statusCode}`);
            });
        });
        
        proxyReq.on('error', (error) => {
            console.error(`    ❌ Error: ${error.message}`);
            res.writeHead(500, { 'Content-Type': 'application/json' });
            res.end(JSON.stringify({
                error: true,
                message: error.message
            }));
        });
        
        // 发送请求体
        if (body) {
            proxyReq.write(body);
        }
        proxyReq.end();
    });
});

// ============================================================
// 启动服务器
// ============================================================
server.listen(PORT, () => {
    console.log('============================================================');
    console.log('Firebase Proxy Server (Node.js)');
    console.log(`启动时间: ${new Date().toISOString()}`);
    console.log(`监听端口: ${PORT}`);
    console.log(`访问地址: http://服务器IP:${PORT}/`);
    console.log('============================================================');
    console.log('');
    console.log('使用方法:');
    console.log(`  状态检查: curl http://服务器IP:${PORT}/`);
    console.log(`  登录测试: POST http://服务器IP:${PORT}/v1/accounts:signInWithPassword?key=xxx`);
    console.log('');
    console.log('等待请求...');
    console.log('');
});

// 优雅关闭
process.on('SIGINT', () => {
    console.log('\n服务器关闭中...');
    server.close(() => {
        console.log('服务器已关闭');
        process.exit(0);
    });
});
