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

    async googleAuth(origin: string) {
        const redirectURL = origin + "/auth/google/callback";
        const response = await this.axiosClient.get<{authorize_url: string, pkce_code_verifier: string}>("/auth/google?redirect_url=" + redirectURL);
        return response.data;
    }

    async googleAuthCallback(
        state: string,
        code: string,
        pkce_code_verifier: string,
        origin: string
        ) {
        const redirectURL = origin + "/auth/google/callback";


        const response = await this.axiosClient.get<{ token: string }>("/auth/google/callback?state=" + state + "&code=" + code + "&pkce_code_verifier=" + pkce_code_verifier + "&redirect_url=" + redirectURL);
        return response.data.token;
    }

}

export const accountService = new AccountService(config.accountURL);