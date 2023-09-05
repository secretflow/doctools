import zod from 'zod';

const SidebarItemDoc = zod.object({
  type: zod.literal('doc'),
  id: zod.string(),
  label: zod.string(),
});

const SidebarItemLink = zod.object({
  type: zod.literal('link'),
  href: zod.string(),
  label: zod.string(),
});

const sidebarCategory = zod.object({
  type: zod.literal('category'),
  label: zod.string(),
  link: SidebarItemDoc.or(zod.null()).default(null).optional(),
});

const SidebarItemCategory: zod.ZodType<SidebarItemCategory> = sidebarCategory.extend({
  items: zod.lazy(() => SidebarItem.array()),
});

const SidebarItem = SidebarItemDoc.or(SidebarItemLink).or(SidebarItemCategory);

const Sidebar = zod.array(SidebarItem);

export const Manifest = zod.object({
  version: zod.literal('1'),
  sidebar: Sidebar,
});

export type SidebarItemDoc = zod.infer<typeof SidebarItemDoc>;
export type SidebarItemLink = zod.infer<typeof SidebarItemLink>;
export type SidebarItemCategory = zod.infer<typeof sidebarCategory> & {
  items: SidebarItem[];
};
export type SidebarItem = zod.infer<typeof SidebarItem>;
export type Sidebar = zod.infer<typeof Sidebar>;
export type Manifest = zod.infer<typeof Manifest>;
