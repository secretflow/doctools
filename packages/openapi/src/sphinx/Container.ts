import styled from 'styled-components';

export const Container = styled.div`
  background-color: #fefefe;
  padding: 16px;
  border-radius: 8px;

  display: flex;
  flex-flow: column nowrap;
  gap: 1rem;
  align-items: stretch;

  p {
    color: #1a1a1a;
  }

  code {
    color: #d63384;
  }

  pre {
    border: none;

    code {
      color: #1a1a1a;
    }
  }
`;
