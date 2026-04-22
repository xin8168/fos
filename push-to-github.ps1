# GitHub Push Script for FOS
cd D:\python\m\fossjy

# Initialize git if needed
git init

# Config
git config user.name "xin8168"
git config user.email "xin8168@github.com"

# Add all files
git add .

# Commit
git commit -m "v0.1.0: FOS神经元控制器初始发布
- 27个核心模块
- 650+单元测试
- 完整的神经元控制架构(感觉/脊髓/运动/末梢)
- 大模型能力延展/脑机接口/具身智能控制
- K8s完整部署栈"

# Add remote
git remote add origin https://github.com/xin8168/fos.git

# Push
git push -u origin main

Write-Host "完成！" -ForegroundColor Green