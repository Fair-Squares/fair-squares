import { createContext, useContext, useReducer, ReactNode } from 'react';
import { CouncilSessionContextState } from './types';
import { InjectedAccountWithMeta } from '@polkadot/extension-inject/types';

const initialSession: CouncilSessionContextState = {
  approved: false,
  selectedProposal: undefined,
  proposals: [],
  role_in_session: '',
  session_closed: false,
  ayes: 0,
  nay: 0,
  council_members: [],
};

type Action =
  | { type: 'SET_APPROVAL'; payload: boolean }
  | { type: 'SET_SELECTED_PROPOSAL'; payload: InjectedAccountWithMeta }
  | { type: 'SET_PROPOSALS'; payload: InjectedAccountWithMeta[] }
  | { type: 'SET_ROLE_IN_SESSION'; payload: string }
  | { type: 'SET_SESSION_CLOSE'; payload: boolean }
  | { type: 'SET_AYES'; payload: number }
  | { type: 'SET_NAY'; payload: number }
  | { type: 'SET_COUNCIL_MEMBERS'; payload: InjectedAccountWithMeta[] };

function reducer(state: CouncilSessionContextState, action: Action): CouncilSessionContextState {
  switch (action.type) {
    case 'SET_APPROVAL':
      return { ...state, approved: action.payload };
    case 'SET_SELECTED_PROPOSAL':
      return { ...state, selectedProposal: action.payload };
    case 'SET_PROPOSALS':
      return{...state,proposals:action.payload};
    case 'SET_SESSION_CLOSE':
      return { ...state, session_closed: action.payload };
    case 'SET_ROLE_IN_SESSION':
      return { ...state, role_in_session: action.payload };
    case 'SET_AYES':
      return { ...state, ayes: action.payload };
    case 'SET_NAY':
      return { ...state, nay: action.payload };
    case 'SET_COUNCIL_MEMBERS':
      return { ...state, council_members: action.payload };
    default:
      return state;
  }
}

type CouncilSessionContextType = CouncilSessionContextState & {
  dispatch1: React.Dispatch<Action>;
};
const CouncilSessionContext = createContext<CouncilSessionContextType>({
  ...initialSession,
  dispatch1: () => {},
});
type Props = {
  children: ReactNode;
};
export function CouncilSessionProvider({ children }: Props) {
  const [{ approved,selectedProposal,proposals, role_in_session, session_closed, ayes, nay, council_members }, dispatch1] =
    useReducer(reducer, initialSession);
  return (
    <CouncilSessionContext.Provider
      value={{
        approved,
        selectedProposal,
        proposals,
        role_in_session,
        session_closed,
        ayes,
        nay,
        council_members,
        dispatch1,
      }}
    >
      {children}
    </CouncilSessionContext.Provider>
  );
}

export const useConcilSessionContext = () => useContext(CouncilSessionContext);
