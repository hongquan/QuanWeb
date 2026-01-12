#!/usr/bin/env python

import asyncio
import re
import tempfile
from pathlib import Path
from typing import Any
from urllib.parse import urlparse
from uuid import UUID

import click
import gel
import httpx
import msgspec

from queries.list_post_containing_imgur_async_edgeql import (
    ListPostContainingImgurResult,
    list_post_containing_imgur,
)

# Type definitions
ImageEntry = dict[str, str | None]
PostReport = dict[str, Any]
ProcessedResults = list[PostReport]

IMGUR_HEADERS: dict[str, str] = {
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

IMGUR_PATTERN = re.compile(r'https?://i\.imgur\.com/[^\s"\')\]>]+')


@click.command()
def main() -> None:
    asyncio.run(process())


async def download_image(
    http_client: httpx.AsyncClient, 
    url: str, 
    imgur_dir: Path, 
    post_index: int, 
    total_posts: int
) -> str | None:
    """Download a single image from Imgur."""
    try:
        click.secho(f'Downloading image ({post_index+1}/{total_posts}): {url}', fg='blue')

        response = await http_client.get(url, headers=IMGUR_HEADERS)
        response.raise_for_status()

        # Extract filename from URL using urllib.parse and pathlib
        parsed_url = urlparse(url)
        filename = Path(parsed_url.path).name
        
        # If no filename or extension, use a default
        if not filename or '.' not in filename:
            filename = 'image.jpg'
        
        # Save image file
        image_path = imgur_dir / filename
        image_path.write_bytes(response.content)
        return filename
    except httpx.RequestError as e:
        click.secho(f'Failed to download {url}: {e}', fg='red')
        return None


async def process() -> None:
    client = gel.create_async_client()
    try:
        post_list: list[ListPostContainingImgurResult] = await list_post_containing_imgur(client)
    finally:
        await client.aclose()

    # Early return if no results
    if not post_list:
        click.secho('No posts found containing imgur links', fg='yellow')
        return

    # Create directories
    base_dir = Path(tempfile.mkdtemp(prefix='post-migration-'))
    imgur_dir = base_dir / 'imgur'
    imgur_dir.mkdir(parents=True, exist_ok=True)

    # Extract imgur URLs from each post
    processed_results: ProcessedResults = []

    for post in post_list:
        body_content = post.body or ''

        # Find all imgur URLs in the body
        imgur_urls: tuple[str, ...] = tuple(IMGUR_PATTERN.findall(body_content))
        
        # Convert URLs to HTTPS
        imgur_urls = tuple(url.replace('http://', 'https://') for url in imgur_urls)

        # Create a new entry with images table
        report: PostReport = {
            'id': post.id,
            'images': tuple({'imgur': url, 'bunny': None} for url in imgur_urls),
        }
        processed_results.append(report)

    # Generate YAML from result using msgspec
    yaml_data: bytes = msgspec.yaml.encode(processed_results)

    # Save YAML file BEFORE downloading images
    yaml_path = base_dir / 'data.yaml'
    yaml_path.write_bytes(yaml_data)

    click.secho(f"YAML data file generated: {yaml_path}", fg='green')
    click.secho(f"Starting image downloads...", fg='green')

    # HTTP client for downloading images with HTTP/2 support
    async with httpx.AsyncClient(headers=IMGUR_HEADERS, http2=True) as http_client:
        for i, (post, report) in enumerate(zip(post_list, processed_results)):
            imgur_urls = tuple(image['imgur'] for image in report['images'])
            
            # Download each image
            downloaded_images: list[str] = []
            for url in imgur_urls:
                filename = await download_image(http_client, url, imgur_dir, i, len(post_list))
                if filename:
                    downloaded_images.append(filename)

    click.secho(f"Files generated in '{base_dir}':", fg='green')
    click.secho(f'  - YAML data: {yaml_path}', fg='green')
    click.secho(f'  - Imgur images: {imgur_dir}', fg='green')
    click.secho(f'Processed {len(processed_results)} posts with Imgur images', fg='green')


if __name__ == '__main__':
    main()
