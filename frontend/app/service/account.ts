import axios, {type AxiosInstance} from 'axios';
import config from "~/service/config";

interface TokenResponse {
    token: string;
}

interface UserResponse {
    id: number;
    username: string;
}

export class AccountService {
    private axiosClient: AxiosInstance;

    constructor(baseURL: string) {
        this.axiosClient = axios.create({
            baseURL: baseURL,
            headers: {'Content-Type': 'application/json'},
        })
    }
    async register(username: string, password: string) {
        const response = await this.axiosClient.post<TokenResponse>("/register", {name: username, password: password});
        return response.data;
    }

    async login(username: string, password: string) {
        const response = await this.axiosClient.post<TokenResponse>("/login", {name: username, password: password});
        return response.data;
    }

    async me(token: string) { // TODO auth token middleware
        const response = await this.axiosClient.get<UserResponse>("/me", {
            headers: {
                Authorization: `Bearer ${token}`,
                'Content-Type': 'application/json',
            }
        });
        return response.data;
    }

    async googleAuth() {
        const response = await this.axiosClient.get<string>("/auth/google?redirect_url=" + location.origin + "/auth/google/callback", );
        return response.data;
    }

}

export const accountService = new AccountService(config.accountURL);