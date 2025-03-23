import axios, {type AxiosInstance} from 'axios';


interface TokenResponse {
    token: string;
}

export class AccountService {
    private axiosClient: AxiosInstance;

    constructor(baseURL: string = 'http://localhost:8080') {
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
}

export const accountService = new AccountService();