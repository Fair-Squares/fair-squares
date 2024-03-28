import { createContext, useContext, useReducer, ReactNode } from 'react';
import { AccountContextState } from './types';
import BN from 'bn.js';

const initialAccount: AccountContextState = {
  address: '',
  role: [],
  balance: undefined,
};

type Action =
  | { type: 'SET_ADDRESS'; payload: string }
  | { type: 'SET_ROLES'; payload: string[] }
  | { type: 'SET_BALANCE'; payload: BN };

function reducer(state: AccountContextState, action: Action): AccountContextState {
  switch (action.type) {
    case 'SET_ADDRESS':
      return { ...state, address: action.payload };
    case 'SET_ROLES':
      return { ...state, role: action.payload };
    case 'SET_BALANCE':
      return { ...state, balance: action.payload };

    default:
      return state;
  }
}

type AccountContextType = AccountContextState & {
  dispatch0: React.Dispatch<Action>;
};
const AccountContext = createContext<AccountContextType>({
  ...initialAccount,
  dispatch0: () => {},
});
type Props = {
  children: ReactNode;
};
export function AccountProvider({ children }: Props) {
  const [{ address, role, balance }, dispatch0] = useReducer(reducer, initialAccount);
  return (
    <AccountContext.Provider
      value={{
        address,
        role,
        balance,
        dispatch0,
      }}
    >
      {children}
    </AccountContext.Provider>
  );
}
export const useAccountContext = () => useContext(AccountContext);
