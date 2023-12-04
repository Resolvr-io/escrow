import React, {
  createContext,
  useContext,
  useState,
  FunctionComponent,
} from "react";

type User = {
  id: string;
  name: string;
} | null;

type AuthContextType = {
  user: User;
  login: (userCredentials: any) => void;
  logout: () => void;
};

const AuthContext = createContext<AuthContextType | null>(null);

type Props = {
  children: React.ReactNode;
};

export const AuthProvider: FunctionComponent<Props> = ({ children }) => {
  const [user, setUser] = useState<User>(null);

  const login = (userCredentials: any) => {
    // TODO: Replace 'any' with a specific type for 'userCredentials'.
    // Implement your login logic here
    setUser({ id: "1", name: "John Doe" }); // Dummy user
  };

  const logout = () => {
    // TODO: Implement login logic.
    setUser(null);
  };

  return (
    <AuthContext.Provider value={{ user, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
};
