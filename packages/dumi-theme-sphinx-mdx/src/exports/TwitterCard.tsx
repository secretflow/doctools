import { Helmet, useIntl, useRouteMeta } from 'dumi';

// straight from dumi
export function TwitterCard({ children }: React.PropsWithChildren) {
  const intl = useIntl();
  const { frontmatter: fm } = useRouteMeta();
  return (
    <Helmet>
      <html lang={intl.locale.replace(/-.+$/, '')} />
      {fm.title && <title>{fm.title}</title>}
      {/* TODO: include site suffix */}
      {fm.title && <meta property="og:title" content={fm.title} />}
      {fm.description && <meta name="description" content={fm.description} />}
      {fm.description && <meta property="og:description" content={fm.description} />}
      {fm.keywords && <meta name="keywords" content={fm.keywords.join(',')} />}
      {fm.keywords && <meta property="og:keywords" content={fm.keywords.join(',')} />}
      {children}
    </Helmet>
  );
}
