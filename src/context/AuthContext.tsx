import React, { createContext, useContext, useState, FunctionComponent } from 'react';

// Define a type for the user object
type User = {
  id: string;
  name: string;
} | null;

// Define a type for the context value
type AuthContextType = {
  user: User;
  login: (userCredentials: any) => void; // Define a more specific type for userCredentials as needed
  logout: () => void;
};

// Create the Auth context with a default value of null
const AuthContext = createContext<AuthContextType | null>(null);

type Props = {
  children: React.ReactNode;
};

export const AuthProvider: FunctionComponent<Props> = ({ children }) => {
  const [user, setUser] = useState<User>(null);

  const login = (userCredentials: any) => { // Replace 'any' with a more specific type for userCredentials
    // Implement your login logic here
    setUser({ id: '1', name: 'John Doe' }); // Dummy user
  };

  const logout = () => {
    // Implement your logout logic here
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
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};

