import { useFragmentFocus } from '../common/positioning.js';

export const Target = ({ id, ...props }: React.ComponentProps<'span'>) => {
  useFragmentFocus(id);
  return <span id={id} {...props} />;
};
