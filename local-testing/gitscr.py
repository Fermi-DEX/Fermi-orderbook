import requests
from bs4 import BeautifulSoup

# URL of the GitHub repository
url = "https://github.com/openbook-dex/openbook-v2/tree/master/programs/openbook-v2/src/state/orderbook"

# Function to get the list of file URLs from the GitHub page
def get_file_urls(url):
    response = requests.get(url)
    if response.status_code != 200:
        return []

    soup = BeautifulSoup(response.text, 'html.parser')
    file_links = soup.find_all('a', class_='js-navigation-open Link--primary')

    raw_urls = []
    base_raw_url = "https://raw.githubusercontent.com"
    for link in file_links:
        href = link.get('href')
        if href.endswith(".rs"):
            raw_url = base_raw_url + href.replace('/blob', '')
            raw_urls.append(raw_url)

    return raw_urls

# Get raw file URLs
file_urls = get_file_urls(url)

# Function to download and concatenate file contents
def download_and_combine_files(file_urls):
    combined_content = ""
    for file_url in file_urls:
        response = requests.get(file_url)
        if response.status_code == 200:
            combined_content += response.text + "\n\n"  # Add a new line between files for clarity

    return combined_content

# Download and combine the file contents
combined_file_content = download_and_combine_files(file_urls)
print(combined_file_content[:500])  # Displaying the first 500 characters for verification
