#!/usr/bin/env python

import asyncio
import re
import tempfile
from pathlib import Path

import click
import gel
import httpx
import msgspec

from queries.list_post_containing_imgur_async_edgeql import list_post_containing_imgur


@click.command()
def main():
    asyncio.run(process())


async def process():
    client = gel.create_async_client()
    try:
        result_list = await list_post_containing_imgur(client)
    finally:
        await client.aclose()

    # Early return if no results
    if not result_list:
        click.secho('No posts found containing imgur links', fg='yellow')
        return

    # Create directories
    base_dir = Path(tempfile.mkdtemp(dir='/tmp'))
    imgur_dir = base_dir / 'imgur'
    imgur_dir.mkdir(parents=True, exist_ok=True)

    # Base headers to fake being a browser
    base_headers = {
        'User-Agent': 'Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0',
        'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8',
        'Accept-Language': 'vi,en-US;q=0.7,en;q=0.3',
        'Accept-Encoding': 'gzip, deflate, br, zstd',
        'Connection': 'keep-alive',
        'Upgrade-Insecure-Requests': '1',
        'Sec-Fetch-Dest': 'document',
        'Sec-Fetch-Mode': 'navigate',
        'Sec-Fetch-Site': 'none',
        'Pragma': 'no-cache',
        'Cache-Control': 'no-cache',
    }

    # Extract imgur URLs from each post
    processed_results = []
    # Improved pattern to capture direct image URLs
    imgur_pattern = re.compile(r'https?://i\.imgur\.com/[^\s"\')\]>]+')

    # HTTP client for downloading images with HTTP/2 support
    async with httpx.AsyncClient(headers=base_headers, http2=True) as http_client:
        for obj in result_list:
            body_content = obj.body or ''

            # Find all imgur URLs in the body
            imgur_urls = tuple(imgur_pattern.findall(body_content))
            
            # Convert URLs to HTTPS
            imgur_urls = tuple(url.replace('http://', 'https://') for url in imgur_urls)

            # Download each image
            downloaded_images = []
            for url in imgur_urls:
                try:
                    click.secho(f'Downloading image: {url}', fg='blue')
                    # Use exact headers from browser
                    imgur_headers = {
                        'User-Agent': 'Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0',
                        'Accept': 'image/avif,image/webp,image/png,image/svg+xml,image/*;q=0.8,*/*;q=0.5',
                        'Accept-Language': 'vi,en-US;q=0.7,en;q=0.3',
                        'Accept-Encoding': 'gzip, deflate, br, zstd',
                        'Connection': 'keep-alive',
                        'Referer': 'https://quan.hoabinh.vn/',
                        'Sec-Fetch-Dest': 'image',
                        'Sec-Fetch-Mode': 'no-cors',
                        'Sec-Fetch-Site': 'cross-site',
                        'Pragma': 'no-cache',
                        'Cache-Control': 'no-cache',
                    }

                    response = await http_client.get(url, headers=imgur_headers)
                    response.raise_for_status()

                    # Extract filename from URL
                    filename = url.split('/')[-1].split('?')[0] or 'image.jpg'
                    if '.' not in filename:
                        filename += '.jpg'

                    # Save image file
                    image_path = imgur_dir / filename
                    image_path.write_bytes(response.content)
                    downloaded_images.append(filename)
                except httpx.RequestError as e:
                    click.secho(f'Failed to download {url}: {e}', fg='red')

            # Create a new entry with images table
            processed_obj = {
                'id': obj.id,
                'images': tuple({'imgur': url, 'bunny': None} for url in imgur_urls),
                'downloaded_images': tuple(downloaded_images),
            }
            processed_results.append(processed_obj)

    # Generate YAML from result using msgspec
    yaml_data = msgspec.yaml.encode(tuple(processed_results))

    # Save YAML file
    yaml_path = base_dir / 'data.yaml'
    yaml_path.write_bytes(yaml_data)

    click.secho(f"Files generated in '{base_dir}':", fg='green')
    click.secho(f'  - YAML data: {yaml_path}', fg='green')
    click.secho(f'  - Imgur images: {imgur_dir}', fg='green')
    click.secho(f'Processed {len(processed_results)} posts with Imgur images', fg='green')


if __name__ == '__main__':
    main()
