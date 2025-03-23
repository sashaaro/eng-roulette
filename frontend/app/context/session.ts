import {type ContextType, createContext, type ReactNode, useEffect, useState} from "react";

interface User {
    id: number;
    username: string;
}

interface AuthContextType {
    user: User | null
    signIn: (email: string, password: string) => Promise<void>
    signUp: (email: string, password: string, name?: string) => Promise<void>
    signOut: () => Promise<void>
}

export function AuthProvider({children}: { children: ReactNode }) {
    const [user, setUser] = useState<User | null>(null)

    useEffect(() => {
        //checkAuth()
    }, [])
}

export const SessionContext = createContext<AuthContextType|undefined>(undefined)