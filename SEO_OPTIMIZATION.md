# 隐语官网SEO优化变更文档

> **⚠️ 重要说明**: 本文档记录了基于SecretFlow DocTools的SEO优化改造内容，专为隐语官网SEO需求而开发。

## 📋 变更概览

本次更新主要包含以下SEO优化功能：

### 🎯 新增SEO组件

#### 1. CanonicalLink组件 (`src/js/browser/layout/CanonicalLink.tsx`)
- **功能**: 为相同内容页面添加标准链接标签
- **作用**: 避免搜索引擎对重复内容进行降权处理
- **实现**: 自动检测版本差异，为旧版本页面添加canonical链接

#### 2. HTMLDescription组件 (`src/js/browser/layout/HTMLDescription.tsx`)
- **功能**: 动态提取页面描述信息
- **作用**: 优化meta description，提升搜索引擎摘要质量
- **实现**: 支持异步获取页面描述，提供默认值回退

#### 3. HTMLKeywords组件 (`src/js/browser/layout/HTMLKeywords.tsx`)
- **功能**: 动态提取页面关键词
- **作用**: 优化meta keywords，提升搜索引擎相关性
- **实现**: 集成国际化支持，支持自定义关键词提取

### 🔧 现有组件增强

#### 1. 链接优化增强
- **文件**: `src/js/browser/page/components/intrinsic.tsx`
- **变更**: 所有外部`<a>`标签自动添加`rel="nofollow"`属性
- **作用**: 防止PageRank权重流失到外部站点

#### 2. 图片优化增强
- **文件**: `src/js/browser/page/components/ImageViewer.tsx`
- **变更**: 所有图片展示组件统一添加描述性alt属性
- **作用**: 提升图片内容的可访问性和搜索引擎理解度

### 🌐 国际化支持

#### 1. 多语言SEO文案
- **文件**: `src/js/browser/locales/*/messages.po`
- **新增**: "SecretFlow Docs"国际化文案
- **支持**: 中英文双语环境

### 📊 技术实现细节

#### 新增SEO API

```typescript
// SEO关键词提取（新增）
useSiteKeywords(page, repo) // 动态提取关键词
getSiteDescription(page, repo) // 动态提取描述
```

#### 版本管理工具（现有功能）
项目使用现有的版本管理功能：
- `findStableVersion({ sorted })` - 从版本列表中查找最稳定版本
- `ltBaseVersion(a, b)` - 比较两个PEP440版本号

#### 组件集成

```tsx
// 在应用中使用SEO组件
import { CanonicalLink } from './layout/CanonicalLink';
import { HTMLDescription } from './layout/HTMLDescription';
import { HTMLKeywords } from './layout/HTMLKeywords';

// 在页面头部集成
<Helmet>
  <CanonicalLink />
  <HTMLDescription />
  <HTMLKeywords />
</Helmet>
```

### 🚀 部署指南

#### 构建命令
```bash
# 安装依赖
pnpm install

# 构建项目
pnpm nx build:lib

# 提取国际化文案
pnpm nx run doctools:lingui:extract

# 编译国际化
pnpm nx run doctools:babel:update
```

### 📈 SEO效果预期

1. **TDK优化**: 提升页面标题、描述、关键词的相关性
2. **链接优化**: 防止权重流失，提升内部链接权重
3. **图片优化**: 提升图片搜索排名和无障碍访问
4. **重复内容处理**: 避免搜索引擎惩罚

### 🔍 验证方法

1. **页面源码检查**: 查看页面head标签中的meta信息
2. **搜索引擎工具**: 使用Google Search Console验证
3. **SEO检测工具**: 使用在线SEO检测工具验证

### 📞 技术支持

- **需求跟踪**: [项目需求](https://project.alipay.com/?openWorkItemId=2025121200112686515&status=status)
- **官网地址**: [隐语官网](https://www.secretflow.org.cn/zh-CN)

---

<div align="center">
  <sub>为隐语官网SEO优化定制开发 | 蚂蚁集团 隐语团队</sub>
</div>
