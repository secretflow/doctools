import styled from 'styled-components';

export const FancyOutput = styled.div`
  > div {
    // FIXME: hardcoded
    margin-inline-start: calc(80px + 0.8rem);

    @media screen and (max-width: 991px) {
      margin-inline-start: 0;
    }

    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 0.5rem;

    table {
      font-size: 0.9rem;
    }

    th {
      padding: 2px 6px;
    }
  }
`;
