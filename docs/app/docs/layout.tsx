import { source } from '@/lib/source';
import { DocsLayout } from 'fumadocs-ui/layouts/docs';
import { baseOptions } from '@/lib/layout.shared';

export default function Layout({ children }: LayoutProps<'/docs'>) {
  return (
    <DocsLayout
      tree={source.getPageTree()}
      {...baseOptions()}
      sidebar={{
        tabs: [
          {
            title: 'v0.1.2',
            description: 'Current release',
            url: '/docs',
          },
        ],
      }}
    >
      {children}
    </DocsLayout>
  );
}
