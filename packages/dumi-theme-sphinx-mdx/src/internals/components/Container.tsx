import DumiContainer from 'dumi/theme-default/builtins/Container';

const Component = DumiContainer as unknown as React.FC<{ id?: string }>;

export const Container = ({ id, ...props }: React.ComponentProps<typeof Component>) =>
  id ? (
    <div id={id}>
      <Component {...props} />
    </div>
  ) : (
    <Component {...props} />
  );
