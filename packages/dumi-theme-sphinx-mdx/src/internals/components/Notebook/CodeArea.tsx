import styled from 'styled-components';

const Cell = styled.div`
  display: flex;

  flex-flow: row nowrap;
  justify-content: stretch;
  align-items: baseline;

  gap: 0.8rem;

  @media screen and (max-width: 991px) {
    flex-flow: column nowrap;
    min-width: 0;
  }
`;

const CellContent = styled.div`
  flex: 1 1 auto;

  display: flex;
  flex-flow: column;

  max-width: 100%;
  min-width: 0;
  overflow: scroll;

  border: 1px solid #ebebeb;

  pre {
    padding: 12px 18px;
    font-size: 0.9rem;
    line-height: 1.5;
  }
`;

const Prompt = styled.code<{ type?: string | undefined }>`
  text-align: right;

  font-size: 0.9rem;

  flex: 0 0 auto;

  // FIXME: hardcoded
  min-width: 80px;

  @media screen and (max-width: 991px) {
    min-width: unset;
  }

  // FIXME: hardcoded, important
  color: ${({ type, theme }) =>
    type === 'input' ? '#5CADF1' : theme.colors.strong} !important;
`;

export const CodeArea = ({
  prompt,
  children,
  type,
}: {
  prompt?: string;
  children?: React.ReactNode;
  type?: string;
}) => {
  return (
    <Cell>
      <Prompt type={type}>
        {type === 'input' ? 'In' : 'Out'} {prompt}
      </Prompt>
      <CellContent>{children}</CellContent>
    </Cell>
  );
};
