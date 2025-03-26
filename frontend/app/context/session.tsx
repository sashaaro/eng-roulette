import {type ContextType, createContext, type ReactNode, useContext, useEffect, useState} from "react";
import {accountService} from "~/service/account";

export interface User {
    id: number;
    username: string;
    token: string;
}

interface AuthContextType {
    user: User | null
    setUser: (user: User|null) => void
}

export const SessionContext = createContext<AuthContextType|undefined>(undefined)

export function AuthProvider({children}: { children: ReactNode }) {
    const [user, setUser] = useState<User | null>(null)

    useEffect(() => {
        const token = localStorage.getItem("session")
        if (token) {
            // TODO loading
            accountService.me(token).then(user => {
                setUser({
                    username: user.username,
                    id: user.id,
                    token: token,
                })
            })
        }
        //checkAuth()
    }, [])
    useEffect(() => {
        localStorage.setItem("session", user?.token || "")
    }, [user]);

    const ctx: AuthContextType = {user: user, setUser}

    return (
        <SessionContext.Provider value={ctx}>{children}
        </SessionContext.Provider>
    )
}

export function useAuth() {
    const context = useContext(SessionContext)
    if (context === undefined) {
        throw new Error('useAuth must be used within an AuthProvider')
    }
    return context
}