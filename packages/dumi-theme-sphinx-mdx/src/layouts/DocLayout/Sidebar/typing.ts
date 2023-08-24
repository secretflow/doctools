export type SidebarItem = {
  key: string;
  selectable?: boolean; // visitable
  title?: React.ReactNode;
  children: SidebarItem[];
};
