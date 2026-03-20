# Node.js Firebase 代理服务器

部署在国外服务器上，国内 Python 脚本通过此代理访问 Firebase API。

## 优点

- **无请求限制**：不像 Cloudflare Workers 有每日 100,000 次限制
- **零依赖**：纯 Node.js 原生模块，无需 npm install
- **简单部署**：一个文件即可运行

---

## 部署步骤

### 1. 上传文件到国外服务器

将 `server.js` 和 `start.sh` 上传到服务器，例如 `/root/firebase-proxy/`

### 2. 添加执行权限

```bash
chmod +x start.sh
```

### 3. 前台运行（测试用）

```bash
node server.js
# 或指定端口
PORT=8080 node server.js
```

### 4. 后台运行（生产用）

```bash
# 使用 nohup
nohup node server.js > output.log 2>&1 &

# 查看日志
tail -f output.log

# 查看进程
ps aux | grep server.js

# 停止
pkill -f server.js
```

### 5. 使用 PM2 守护进程（推荐）

```bash
# 安装 PM2
npm install -g pm2

# 启动
pm2 start server.js --name firebase-proxy

# 查看状态
pm2 status

# 查看日志
pm2 logs firebase-proxy

# 停止
pm2 stop firebase-proxy

# 设置开机自启
pm2 startup
pm2 save
```

---

## 配置防火墙

```bash
# CentOS
firewall-cmd --zone=public --add-port=3000/tcp --permanent
firewall-cmd --reload

# Ubuntu
ufw allow 3000
```

---

## 测试代理

### 测试状态

```bash
curl http://服务器IP:3000/
```

应返回：
```json
{"status":"ok","message":"Firebase Proxy is running (Node.js)"}
```

### 测试登录

```bash
curl -X POST "http://服务器IP:3000/v1/accounts:signInWithPassword?key=AIzaSyDsOl-1XpT5err0Tcnx8FFod1H8gVGIycY" \
  -H "Content-Type: application/json" \
  -d '{"email":"RobinBrown1266@outlook.com","password":"Qwer123456++","returnSecureToken":true}'
```

---

## 配置 Python 脚本

修改 `test_login.py` 中的代理地址：

```python
PROXY_WORKER_URL = "http://你的服务器IP:3000"
```

---

## 使用 HTTPS（可选）

如需 HTTPS，可使用 Nginx 反向代理：

```nginx
server {
    listen 443 ssl;
    server_name your-domain.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

---

## 作者信息

- **微信**：1837620622（传康 kk）
- **邮箱**：2040168455@qq.com
- **咸鱼/B站**：万能程序员
