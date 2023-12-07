import {
  GithubOutlined as GitHubOutlined,
  DownloadOutlined,
  MessageOutlined,
  ClockCircleOutlined,
} from '@ant-design/icons';
import { Tooltip, Typography, ConfigProvider } from 'antd';
import { formatRelative } from 'date-fns';
import { zhCN } from 'date-fns/locale';
import { useRouteMeta } from 'dumi';
import newGitHubIssueURL from 'new-github-issue-url';

export function GitHubPortal() {
  const frontmatter = useRouteMeta().frontmatter;
  const gitOriginURL: string | undefined = frontmatter['git_origin_url'];
  const gitDownloadURL: string | undefined = frontmatter['git_download_url'];
  const newIssueURL = (() => {
    if (!gitOriginURL) {
      return undefined;
    }
    const template = `
### 问题描述

<!-- 请在此描述你遇到的文档问题 -->

### 技术信息

- 页面 ${window.location.href}
- 源码 ${gitOriginURL}
- 浏览器 \`${navigator.userAgent}\`
    `.trim();
    return newGitHubIssueURL({
      title: '文档问题',
      user: frontmatter['git_owner'],
      repo: frontmatter['git_repo'],
      body: template,
    });
  })();
  const gitTimestamp: Date | undefined =
    frontmatter['git_timestamp'] && new Date(frontmatter['git_timestamp']);
  const gitCommit: string | undefined = frontmatter['git_commit'];
  return (
    <>
      {gitOriginURL ? (
        <>
          <Typography.Link
            type="secondary"
            href={frontmatter['git_origin_url']}
            target="_blank"
          >
            <GitHubOutlined /> 在 GitHub 上打开
          </Typography.Link>
        </>
      ) : null}
      {gitDownloadURL && gitOriginURL?.endsWith('.ipynb') ? (
        <Typography.Link type="secondary" href={gitDownloadURL} download={true}>
          <DownloadOutlined /> 下载 Notebook
        </Typography.Link>
      ) : null}
      {newIssueURL ? (
        <Typography.Link type="secondary" href={newIssueURL} target="_blank">
          <MessageOutlined /> 报告文档问题
        </Typography.Link>
      ) : null}
      {gitCommit && gitTimestamp ? (
        <ConfigProvider>
          <Tooltip title={<code>Revision {gitCommit.slice(0, 7)}</code>}>
            <Typography.Text type="secondary">
              <ClockCircleOutlined /> 最后更新于{' '}
              {formatRelative(gitTimestamp, new Date(), { locale: zhCN })}
            </Typography.Text>
          </Tooltip>
        </ConfigProvider>
      ) : null}
    </>
  );
}
