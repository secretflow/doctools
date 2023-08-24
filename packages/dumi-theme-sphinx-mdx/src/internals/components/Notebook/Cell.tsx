import styled from 'styled-components';

export const Cell = styled.section`
  padding: 0.5rem 1rem;

  display: flex;
  flex-flow: column nowrap;
  gap: 0.5rem;

  @media screen and (max-width: 991px) {
    padding: 0;
  }
`;
